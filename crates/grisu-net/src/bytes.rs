//! Lectura y escritura binaria en little-endian.
//!
//! [`Reader`] nunca hace `panic!`: los datos llegan de la red y pueden ser
//! cualquier cosa, así que cada lectura devuelve un `Result`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// Faltan bytes.
    UnexpectedEnd,
    /// Un valor no es válido (etiqueta desconocida, longitud absurda...).
    Invalid(&'static str),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEnd => write!(f, "datos truncados"),
            Self::Invalid(what) => write!(f, "dato inválido: {what}"),
        }
    }
}

impl std::error::Error for DecodeError {}

#[derive(Clone, Debug, Default)]
pub struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    pub fn u8(&mut self, v: u8) -> &mut Self {
        self.buf.push(v);
        self
    }

    pub fn bool(&mut self, v: bool) -> &mut Self {
        self.u8(u8::from(v))
    }

    pub fn i8(&mut self, v: i8) -> &mut Self {
        self.u8(v as u8)
    }

    pub fn u16(&mut self, v: u16) -> &mut Self {
        self.buf.extend_from_slice(&v.to_le_bytes());
        self
    }

    pub fn u32(&mut self, v: u32) -> &mut Self {
        self.buf.extend_from_slice(&v.to_le_bytes());
        self
    }

    pub fn u64(&mut self, v: u64) -> &mut Self {
        self.buf.extend_from_slice(&v.to_le_bytes());
        self
    }

    pub fn i32(&mut self, v: i32) -> &mut Self {
        self.buf.extend_from_slice(&v.to_le_bytes());
        self
    }

    /// Posiciones del mundo grande: `i64` en Q47.16 (ADR 0002).
    pub fn i64(&mut self, v: i64) -> &mut Self {
        self.buf.extend_from_slice(&v.to_le_bytes());
        self
    }

    /// Bytes sin prefijo de longitud.
    pub fn raw(&mut self, bytes: &[u8]) -> &mut Self {
        self.buf.extend_from_slice(bytes);
        self
    }

    /// Bytes con su longitud delante (u32).
    pub fn bytes(&mut self, bytes: &[u8]) -> &mut Self {
        self.u32(bytes.len() as u32);
        self.raw(bytes)
    }
}

#[derive(Clone, Debug)]
pub struct Reader<'a> {
    data: &'a [u8],
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn remaining(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn raw(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        if self.data.len() < n {
            return Err(DecodeError::UnexpectedEnd);
        }
        let (head, tail) = self.data.split_at(n);
        self.data = tail;
        Ok(head)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        let bytes = self.raw(N)?;
        let mut out = [0; N];
        out.copy_from_slice(bytes);
        Ok(out)
    }

    pub fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.array::<1>()?[0])
    }

    pub fn bool(&mut self) -> Result<bool, DecodeError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DecodeError::Invalid("booleano")),
        }
    }

    pub fn i8(&mut self) -> Result<i8, DecodeError> {
        Ok(self.u8()? as i8)
    }

    pub fn u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    pub fn u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    pub fn u64(&mut self) -> Result<u64, DecodeError> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    pub fn i32(&mut self) -> Result<i32, DecodeError> {
        Ok(i32::from_le_bytes(self.array()?))
    }

    pub fn i64(&mut self) -> Result<i64, DecodeError> {
        Ok(i64::from_le_bytes(self.array()?))
    }

    /// Bytes con su longitud delante (u32). La longitud se comprueba antes
    /// de reservar memoria.
    pub fn bytes(&mut self) -> Result<&'a [u8], DecodeError> {
        let len = self.u32()? as usize;
        self.raw(len)
    }

    /// Falla si sobran bytes: un mensaje bien formado se consume entero.
    pub fn finish(&self) -> Result<(), DecodeError> {
        if self.data.is_empty() {
            Ok(())
        } else {
            Err(DecodeError::Invalid("bytes sobrantes"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_type() {
        let mut w = Writer::new();
        w.u8(7)
            .bool(true)
            .i8(-3)
            .u16(65000)
            .u32(4_000_000_000)
            .u64(u64::MAX - 1)
            .i32(-123_456)
            .i64(i64::MIN + 7)
            .i64(7_499_999 * 65_536)
            .bytes(b"hola");
        let bytes = w.into_bytes();
        let mut r = Reader::new(&bytes);
        assert_eq!(r.u8(), Ok(7));
        assert_eq!(r.bool(), Ok(true));
        assert_eq!(r.i8(), Ok(-3));
        assert_eq!(r.u16(), Ok(65000));
        assert_eq!(r.u32(), Ok(4_000_000_000));
        assert_eq!(r.u64(), Ok(u64::MAX - 1));
        assert_eq!(r.i32(), Ok(-123_456));
        assert_eq!(r.i64(), Ok(i64::MIN + 7));
        assert_eq!(r.i64(), Ok(7_499_999 * 65_536));
        assert_eq!(r.bytes(), Ok(&b"hola"[..]));
        assert_eq!(r.finish(), Ok(()));
    }

    #[test]
    fn truncated_and_absurd_input_are_errors() {
        assert_eq!(Reader::new(&[1]).u16(), Err(DecodeError::UnexpectedEnd));
        // Un i64 necesita 8 bytes, ni uno menos.
        assert_eq!(
            Reader::new(&[1, 2, 3, 4, 5, 6, 7]).i64(),
            Err(DecodeError::UnexpectedEnd)
        );
        assert_eq!(
            Reader::new(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]).i64(),
            Err(DecodeError::UnexpectedEnd)
        );
        assert_eq!(
            Reader::new(&[2]).bool(),
            Err(DecodeError::Invalid("booleano"))
        );
        // Longitud enorme sin datos: error, no reserva de memoria.
        let mut r = Reader::new(&[0xFF, 0xFF, 0xFF, 0xFF, 1, 2]);
        assert_eq!(r.bytes(), Err(DecodeError::UnexpectedEnd));
        assert!(Reader::new(&[0]).finish().is_err());
    }
}
