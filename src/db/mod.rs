use prost::Message;
use prost::bytes::Bytes;

#[derive(Message)]
pub struct Phrase {
    #[prost(message, repeated, tag = "1")]
    pub events: Vec<PhraseEvent>,
}

#[derive(Message)]
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
