use crate::config::RecordInfo;
use crate::SimpleDnsError;
use std::{collections::BTreeMap, str::FromStr, sync::Arc};
use trust_dns_proto::rr::RecordType;
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

pub(crate) fn load_zone(
    origin: &str,
    info: Vec<RecordInfo>,
) -> Result<Box<dyn AuthorityObject>, SimpleDnsError> {
    let mut records: BTreeMap<RrKey, RecordSet> = BTreeMap::new();

    for data in info {
        let record_type = data.record_type;
        let name = get_record_name(&data.name, origin)?;
        let record_set = data.into_record_set(origin)?;

        let key = RrKey::new(name.into(), record_type);
        records.insert(key, record_set);
    }

    let (key, soa_record_set) = get_soa_record(origin)?;
    records.insert(key, soa_record_set);

    let authority =
        InMemoryAuthority::new(Name::from_str(origin)?, records, ZoneType::Primary, false)
            .map_err(SimpleDnsError::ConfigError)?;
    Ok(Box::new(Arc::new(authority)))
}

// TODO: remove async once newer version of ForwardAuthority::try_from_config is released
pub(crate) async fn get_forward_authority(
    origin: Name,
) -> Result<Box<dyn AuthorityObject>, SimpleDnsError> {
    let name_servers = NameServerConfigGroup::google();
    let config = ForwardConfig {
        name_servers,
        options: None,
    };

    let forwarder = ForwardAuthority::try_from_config(origin, ZoneType::Forward, &config)
        .await
        .map_err(SimpleDnsError::ConfigError)?;
    Ok(Box::new(Arc::new(forwarder)))
}

pub(crate) fn get_record_name(name: &str, domain: &str) -> Result<Name, SimpleDnsError> {
    let name = match name {
        "@" => Name::from_str(domain)?,
        _ => {
            let name = format!("{}.{}.", name, domain);
            Name::from_str(&name)?
        }
    };

    Ok(name)
}

pub(crate) fn get_soa_record(domain: &str) -> Result<(RrKey, RecordSet), SimpleDnsError> {
    let name = Name::from_str(domain)?;
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

pub(crate) fn to_rdata(
    record_type: RecordType,
    v: &str,
    origin: &str,
) -> Result<RData, SimpleDnsError> {
    let rdata = match record_type {
        RecordType::A => RData::A(v.parse()?),
        RecordType::AAAA => RData::AAAA(v.parse()?),
        RecordType::CNAME => RData::CNAME(get_record_name(v, origin)?),
        _ => panic!("Not supported record type"),
    };

    Ok(rdata)
}
