use prost::bytes::Bytes;
use prost::Message;

#[derive(Clone, Message)]
pub struct Phrase {
    #[prost(message, repeated, tag = "1")]
    pub events: Vec<PhraseEvent>,
}

#[derive(Clone, Message)]
pub struct PhraseEvent {
    #[prost(uint64, tag = "1")]
    pub timestamp: u64,
    #[prost(bytes, tag = "2")]
    pub data: Bytes,
}

pub fn write_phrase(phrase: &Phrase) -> anyhow::Result<()> {
    let path = "./data/phrase.dat";
    let bytes = phrase.encode_to_vec();
    fs_err::write(path, bytes)?;
    Ok(())
}
