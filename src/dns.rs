use std::{collections::BTreeMap, str::FromStr, sync::Arc};

pub use trust_dns_proto::rr::RecordType;
use trust_dns_proto::rr::{rdata::SOA, DNSClass, RData, Record, RecordSet};
use trust_dns_resolver::{config::NameServerConfigGroup, Name};
use trust_dns_server::{
    authority::{AuthorityObject, ZoneType},
    client::rr::RrKey,
    store::{
        forwarder::{ForwardAuthority, ForwardConfig},
        in_memory::InMemoryAuthority,
    },
};

use crate::config::RecordInfo;

pub(crate) fn load_zone(
    origin: &str,
    info: Vec<RecordInfo>,
) -> Result<Box<dyn AuthorityObject>, String> {
    let mut records: BTreeMap<RrKey, RecordSet> = BTreeMap::new();

    for data in info {
        let record_type = data.record_type;
        let name = get_record_name(&data.name, origin)?;
        let record_set = data.to_record_set(origin)?;

        let key = RrKey::new(name.into(), record_type);
        records.insert(key, record_set);
    }

    let (key, soa_record_set) = get_soa_record(origin)?;
    records.insert(key, soa_record_set);

    let authority =
        InMemoryAuthority::new(Name::from_str(origin)?, records, ZoneType::Primary, false)?;
    Ok(Box::new(Arc::new(authority)))
}

// TODO: remove async once newer version of ForwardAuthority::try_from_config is released
pub(crate) async fn get_forward_authority(
    origin: Name,
) -> Result<Box<dyn AuthorityObject>, String> {
    let name_servers = NameServerConfigGroup::google();
    let config = ForwardConfig {
        name_servers,
        options: None,
    };

    let forwarder = ForwardAuthority::try_from_config(origin, ZoneType::Forward, &config).await?;
    Ok(Box::new(Arc::new(forwarder)))
}

pub(crate) fn get_record_name(name: &str, domain: &str) -> Result<Name, String> {
    match name {
        "@" => Name::from_str(domain).map_err(|e| e.to_string()),
        _ => {
            let name = format!("{}.{}.", name, domain);
            Name::from_str(&name).map_err(|e| e.to_string())
        }
    }
}

pub(crate) fn get_soa_record(domain: &str) -> Result<(RrKey, RecordSet), String> {
    let name = Name::from_str(domain).map_err(|e| e.to_string())?;
    let record_type = RecordType::SOA;
    let mut rr_set = RecordSet::new(&name, record_type, 0);

    let insert = Record::new()
        .set_name(name.clone())
        .set_ttl(3600)
        .set_rr_type(RecordType::SOA)
        .set_dns_class(DNSClass::IN)
        .set_data(Some(RData::SOA(SOA::new(
            Name::from_str("sns.dns.icann.org.").unwrap(),
            Name::from_str("noc.dns.icann.org.").unwrap(),
            20,
            7200,
            600,
            3600000,
            60,
        ))))
        .clone();

    rr_set.insert(insert, 0);
    let key = RrKey::new(name.into(), RecordType::SOA);
    Ok((key, rr_set))
}

pub(crate) fn to_rdata(record_type: RecordType, v: &str, origin: &str) -> Result<RData, String> {
    let rdata = match record_type {
        RecordType::A => RData::A(v.parse().map_err(|_| "Failed to parse ipv4 addr")?),
        RecordType::AAAA => RData::AAAA(v.parse().map_err(|_| "Failed to parse ipv6 addr")?),
        RecordType::CNAME => RData::CNAME(get_record_name(v, origin)?),
        _ => panic!("Not supported record type"),
    };

    Ok(rdata)
}
