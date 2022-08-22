use crate::dns::{get_forward_authority, get_record_name, load_zone, to_rdata};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, net::SocketAddr, str::FromStr};
use trust_dns_proto::rr::RecordType;
use trust_dns_proto::rr::{Record, RecordSet};
use trust_dns_resolver::Name;
use trust_dns_server::authority::Catalog;

const DEFAULT_TTL: u32 = 3600;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub bind: SocketAddr,
    pub domains: BTreeMap<String, Vec<RecordInfo>>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct RecordInfo {
    pub name: String,
    pub records: Vec<String>,
    #[serde(
        rename = "type",
        default = "default_record_type",
        skip_serializing_if = "is_default_record_type"
    )]
    pub record_type: RecordType,
}

pub fn default_record_type() -> RecordType {
    RecordType::A
}

pub fn is_default_record_type(record_type: &RecordType) -> bool {
    *record_type == default_record_type()
}

impl RecordInfo {
    pub fn new(name: &str, records: &[&str], record_type: RecordType) -> Self {
        Self {
            name: name.to_string(),
            records: records.iter().map(|s| s.to_string()).collect(),
            record_type,
        }
    }

    pub fn to_record_set(self, origin: &str) -> Result<RecordSet, String> {
        let name = get_record_name(&self.name, origin)?;
        let record_type = self.record_type;

        let mut record_set = RecordSet::with_ttl(name.clone(), record_type, DEFAULT_TTL);
        for r in self.records {
            let rdata = to_rdata(record_type, &r, origin)?;
            let record = Record::from_rdata(name.clone(), DEFAULT_TTL, rdata);
            record_set.insert(record, 0);
        }

        Ok(record_set)
    }
}

impl Config {
    pub async fn load_catalog(self) -> Result<Catalog, String> {
        let mut catalog = Catalog::new();
        for (domain, records) in self.domains {
            let name = Name::from_str(&domain).map_err(|e| e.to_string())?;
            let authority = load_zone(&domain, records)?;

            catalog.upsert(name.into(), authority);
        }

        let origin = Name::from_str(".")?;
        let authority = get_forward_authority(origin.clone()).await?;
        catalog.upsert(origin.into(), authority);

        Ok(catalog)
    }
}
