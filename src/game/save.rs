//! Partida guardada: un archivo binario pequeñísimo (unos cientos de bytes).
//!
//! Formato: `PKGM` + versión + longitud + datos + CRC32 de los datos. Los números van como
//! enteros de longitud variable (LEB128): casi todos ocupan 1 o 2 bytes. Las banderas
//! (entrenadores vencidos, objetos recogidos, Pokédex) van en bits.
//!
//! Se escribe en un archivo temporal, se fuerza a disco y se renombra encima del bueno:
//! si se corta la luz a mitad, la partida anterior sigue intacta. Todo eso ocurre en un hilo
//! aparte para que el juego no se congele ni un fotograma.

use std::path::PathBuf;

use bevy::prelude::*;

use crate::pokemon::{Creature, MAX_LEVEL, Party, SPECIES};
use crate::game::items::{Bag, ITEMS};
use crate::world::kanto::MAPS;
use crate::game::progress::{Flags, Place, Progress};

const MAGIC: &[u8; 4] = b"PKGM";
const VERSION: u8 = 1;

/// Todo lo que se guarda.
#[derive(Clone, PartialEq, Debug)]
pub struct SaveData {
    pub progress: Progress,
    pub party: Vec<Creature>,
    pub bag: Bag,
    pub companion: Option<usize>,
    pub place: Place,
    pub facing: IVec2,
}

/// Dónde se guarda: `$KANTO3D_SAVE`, o `~/.local/share/kanto3d/partida.sav`.
pub fn save_path() -> PathBuf {
    if let Ok(path) = std::env::var("KANTO3D_SAVE") {
        return path.into();
    }
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .or_else(|| std::env::var_os("APPDATA").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("kanto3d").join("partida.sav")
}

// ---------------------------------------------------------------- escribir

struct Writer(Vec<u8>);

impl Writer {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }

    /// Entero sin signo de longitud variable: 7 bits por byte.
    fn var(&mut self, mut v: u64) {
        loop {
            let byte = (v & 0x7f) as u8;
            v >>= 7;
            if v == 0 {
                self.0.push(byte);
                return;
            }
            self.0.push(byte | 0x80);
        }
    }

    /// Con signo: zigzag (los negativos pequeños también ocupan poco).
    fn int(&mut self, v: i64) {
        self.var(((v << 1) ^ (v >> 63)) as u64);
    }

    fn pos(&mut self, p: IVec2) {
        self.int(p.x as i64);
        self.int(p.y as i64);
    }

    fn place(&mut self, p: Place) {
        self.var(p.map as u64);
        self.pos(p.pos);
    }

    fn flags(&mut self, f: &Flags) {
        // Sin las palabras vacías del final
        let len = f.0.iter().rposition(|w| *w != 0).map_or(0, |i| i + 1);
        self.var(len as u64);
        for w in &f.0[..len] {
            self.var(*w);
        }
    }

    fn creature(&mut self, c: &Creature) {
        self.var(c.species as u64);
        self.var(c.level as u64);
        self.var(c.xp as u64);
        self.var(c.hp.max(0) as u64);
    }
}

pub fn encode(data: &SaveData) -> Vec<u8> {
    let p = &data.progress;
    let mut w = Writer(Vec::with_capacity(256));
    let name = p.name.as_bytes();
    w.var(name.len() as u64);
    w.0.extend_from_slice(name);
    w.var(p.id as u64);
    w.var(p.money as u64);
    w.u8(p.badges);
    w.var(p.seconds as u64);
    w.u8(p.champion as u8);
    w.var(p.repel as u64);
    w.var(p.steps as u64);
    w.place(data.place);
    w.u8(dir_code(data.facing));
    w.place(p.back);
    w.place(p.last_center);
    w.var(data.party.len() as u64);
    for c in &data.party {
        w.creature(c);
    }
    w.var(data.companion.map_or(0, |c| c as u64 + 1));
    let items: Vec<usize> = (0..ITEMS.len()).filter(|&i| data.bag.0[i] > 0).collect();
    w.var(items.len() as u64);
    for i in items {
        w.var(i as u64);
        w.var(data.bag.0[i] as u64);
    }
    for f in [&p.done, &p.taken, &p.seen, &p.caught] {
        w.flags(f);
    }
    w.var(p.boxed.len() as u64);
    for c in &p.boxed {
        w.creature(c);
    }

    let payload = w.0;
    let mut out = Vec::with_capacity(payload.len() + 13);
    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(&payload);
    out.extend_from_slice(&crc32(&payload).to_le_bytes());
    out
}

fn dir_code(d: IVec2) -> u8 {
    match (d.x, d.y) {
        (0, 1) => 0,
        (1, 0) => 1,
        (-1, 0) => 3,
        _ => 2,
    }
}

fn code_dir(c: u8) -> IVec2 {
    [IVec2::Y, IVec2::X, IVec2::NEG_Y, IVec2::NEG_X][(c & 3) as usize]
}

// ---------------------------------------------------------------- leer

#[derive(Debug, PartialEq)]
pub enum LoadError {
    Missing,
    Corrupt(&'static str),
}

struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    fn u8(&mut self) -> Result<u8, LoadError> {
        let (&b, rest) = self.0.split_first().ok_or(LoadError::Corrupt("faltan datos"))?;
        self.0 = rest;
        Ok(b)
    }

    fn var(&mut self) -> Result<u64, LoadError> {
        let mut v = 0u64;
        for shift in (0..64).step_by(7) {
            let b = self.u8()?;
            v |= ((b & 0x7f) as u64) << shift;
            if b & 0x80 == 0 {
                return Ok(v);
            }
        }
        Err(LoadError::Corrupt("número demasiado largo"))
    }

    /// Un número que tiene que ser menor que `max`.
    fn below(&mut self, max: usize) -> Result<usize, LoadError> {
        let v = self.var()?;
        if v as usize >= max { Err(LoadError::Corrupt("valor fuera de rango")) } else { Ok(v as usize) }
    }

    fn int(&mut self) -> Result<i64, LoadError> {
        let v = self.var()?;
        Ok((v >> 1) as i64 ^ -((v & 1) as i64))
    }

    fn pos(&mut self) -> Result<IVec2, LoadError> {
        let (x, y) = (self.int()?, self.int()?);
        if x.abs() > 1000 || y.abs() > 1000 {
            return Err(LoadError::Corrupt("posición imposible"));
        }
        Ok(IVec2::new(x as i32, y as i32))
    }

    fn place(&mut self) -> Result<Place, LoadError> {
        Ok(Place { map: self.below(MAPS.len())?, pos: self.pos()? })
    }

    fn flags(&mut self) -> Result<Flags, LoadError> {
        let len = self.below(4096)?;
        (0..len).map(|_| self.var()).collect::<Result<_, _>>().map(Flags)
    }

    fn creature(&mut self) -> Result<Creature, LoadError> {
        let species = self.below(SPECIES.len())?;
        let level = self.var()?.clamp(1, MAX_LEVEL as u64) as u32;
        let mut c = Creature::new(species, level);
        c.xp = self.var()?.min(u32::MAX as u64) as u32;
        c.hp = (self.var()? as i32).min(c.max_hp());
        Ok(c)
    }
}

pub fn decode(bytes: &[u8]) -> Result<SaveData, LoadError> {
    if bytes.len() < 13 || &bytes[..4] != MAGIC {
        return Err(LoadError::Corrupt("no es una partida de Kanto3D"));
    }
    if bytes[4] != VERSION {
        return Err(LoadError::Corrupt("versión desconocida"));
    }
    let len = u32::from_le_bytes(bytes[5..9].try_into().unwrap()) as usize;
    if bytes.len() != 9 + len + 4 {
        return Err(LoadError::Corrupt("tamaño incorrecto"));
    }
    let payload = &bytes[9..9 + len];
    let crc = u32::from_le_bytes(bytes[9 + len..].try_into().unwrap());
    if crc != crc32(payload) {
        return Err(LoadError::Corrupt("el archivo está dañado"));
    }

    let mut r = Reader(payload);
    let name_len = r.below(64)?;
    if r.0.len() < name_len {
        return Err(LoadError::Corrupt("faltan datos"));
    }
    let name = String::from_utf8(r.0[..name_len].to_vec()).map_err(|_| LoadError::Corrupt("nombre ilegible"))?;
    r.0 = &r.0[name_len..];
    let mut progress = Progress {
        name,
        id: r.var()? as u16,
        money: r.var()?.min(crate::game::progress::MAX_MONEY as u64) as u32,
        badges: r.u8()?,
        seconds: r.var()? as f64,
        champion: r.u8()? != 0,
        repel: r.var()? as u32,
        steps: r.var()? as u32,
        ..Progress::default()
    };
    let place = r.place()?;
    let facing = code_dir(r.u8()?);
    progress.back = r.place()?;
    progress.last_center = r.place()?;
    let party_len = r.below(crate::pokemon::MAX_PARTY + 1)?;
    let party = (0..party_len).map(|_| r.creature()).collect::<Result<Vec<_>, _>>()?;
    if party.is_empty() {
        return Err(LoadError::Corrupt("equipo vacío"));
    }
    let companion = match r.var()? {
        0 => None,
        c => Some((c as usize - 1).min(party.len() - 1)),
    };
    let mut bag = Bag([0; ITEMS.len()]);
    for _ in 0..r.below(ITEMS.len() + 1)? {
        let item = r.below(ITEMS.len())?;
        bag.0[item] = r.var()?.min(crate::game::items::MAX_STACK as u64) as u16;
    }
    progress.done = r.flags()?;
    progress.taken = r.flags()?;
    progress.seen = r.flags()?;
    progress.caught = r.flags()?;
    let boxed = r.below(10_000)?;
    progress.boxed = (0..boxed).map(|_| r.creature()).collect::<Result<_, _>>()?;
    if !r.0.is_empty() {
        return Err(LoadError::Corrupt("sobran datos"));
    }
    Ok(SaveData { progress, party, bag, companion, place, facing })
}

/// CRC-32 (el de zip/png), con la tabla calculada al compilar.
fn crc32(data: &[u8]) -> u32 {
    const TABLE: [u32; 256] = {
        let mut table = [0u32; 256];
        let mut i = 0;
        while i < 256 {
            let mut c = i as u32;
            let mut k = 0;
            while k < 8 {
                c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
                k += 1;
            }
            table[i] = c;
            i += 1;
        }
        table
    };
    !data.iter().fold(!0u32, |c, &b| TABLE[((c ^ b as u32) & 0xff) as usize] ^ (c >> 8))
}

// ---------------------------------------------------------------- disco

/// Escribe de forma atómica: temporal + fsync + renombrar.
pub fn write_atomic(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    std::fs::rename(&tmp, path)
}

pub fn load() -> Result<SaveData, LoadError> {
    match std::fs::read(save_path()) {
        Ok(bytes) => decode(&bytes),
        Err(_) => Err(LoadError::Missing),
    }
}

/// Guardado en curso (en otro hilo).
#[derive(Resource)]
pub struct Saving {
    task: bevy::tasks::Task<Result<usize, String>>,
    pub started: std::time::Instant,
}

/// Empieza a guardar en segundo plano. Lo caro (copiar los datos y codificarlos) son
/// microsegundos; la escritura a disco va en el hilo de E/S.
pub fn start_saving(commands: &mut Commands, data: SaveData) {
    let bytes = encode(&data);
    let path = save_path();
    let task = bevy::tasks::IoTaskPool::get().spawn(async move {
        write_atomic(&path, &bytes).map(|_| bytes.len()).map_err(|e| e.to_string())
    });
    commands.insert_resource(Saving { task, started: std::time::Instant::now() });
}

/// Si el guardado ha terminado: `Some(Ok(bytes))` o `Some(Err(motivo))`.
pub fn poll_saving(saving: &mut Saving) -> Option<Result<usize, String>> {
    if !saving.task.is_finished() {
        return None;
    }
    Some(bevy::tasks::block_on(&mut saving.task))
}

/// Junta lo que hay que guardar desde los recursos del juego.
pub fn snapshot(progress: &Progress, party: &Party, bag: &Bag, companion: Option<usize>, place: Place, facing: IVec2) -> SaveData {
    SaveData { progress: progress.clone(), party: party.0.clone(), bag: bag.clone(), companion, place, facing }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pokemon::*;

    fn sample() -> SaveData {
        let mut progress = Progress { name: "Ñandú".into(), money: 12345, badges: 0b1010_0101, seconds: 3725.4, ..Progress::default() };
        progress.done.set(5);
        progress.done.set(700);
        progress.seen.set(GEODUDE);
        progress.caught.set(PIKACHU);
        progress.taken.set(40);
        progress.boxed.push(Creature::new(ONIX, 33));
        let mut party = vec![Creature::new(CHARMANDER, 17), Creature::new(PIKACHU, 12)];
        party[1].hp = 3;
        party[0].xp = 77;
        let mut bag = Bag::default();
        bag.add(crate::game::items::ULTRA_BALL, 300);
        SaveData { progress, party, bag, companion: Some(1), place: Place { map: 10, pos: IVec2::new(7, -2) }, facing: IVec2::NEG_X }
    }

    #[test]
    fn save_round_trip() {
        let data = sample();
        let bytes = encode(&data);
        let mut expected = data.clone();
        // Los segundos se guardan enteros
        expected.progress.seconds = expected.progress.seconds.floor();
        assert_eq!(decode(&bytes).unwrap(), expected);
        assert!(bytes.len() < 200, "la partida ocupa {} bytes", bytes.len());
    }

    #[test]
    fn damaged_saves_are_rejected() {
        let mut bytes = encode(&sample());
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0x10;
        assert!(matches!(decode(&bytes), Err(LoadError::Corrupt(_))));
        assert!(decode(b"hola").is_err());
        let ok = encode(&sample());
        assert!(decode(&ok[..ok.len() - 1]).is_err());
    }

    #[test]
    fn atomic_write_replaces_the_file() {
        let dir = std::env::temp_dir().join(format!("kanto3d-test-{}", std::process::id()));
        let path = dir.join("partida.sav");
        write_atomic(&path, b"uno").unwrap();
        write_atomic(&path, b"dos").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"dos");
        assert!(!path.with_extension("tmp").exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn crc_matches_the_standard() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }
}
