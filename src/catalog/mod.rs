pub mod agencies;
pub mod forms;
pub mod phrases;
pub mod playbooks;
pub mod routes;
pub mod shelters;
pub mod supplies;
pub mod zones;

#[derive(Clone, Copy, Debug)]
pub struct ShelterProfile {
    pub id: u16,
    pub region: u16,
    pub capacity: u16,
    pub baseline_staff: u16,
    pub flags: u16,
    pub name: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct SupplyProfile {
    pub sku: u16,
    pub family: u16,
    pub units_per_case: u16,
    pub cold_chain: bool,
    pub name: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct AgencyProfile {
    pub id: u16,
    pub mutual_aid_zone: u16,
    pub dispatch_weight: u16,
    pub name: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct RouteProfile {
    pub id: u16,
    pub from_zone: u16,
    pub to_zone: u16,
    pub winter_minutes: u16,
    pub flood_minutes: u16,
    pub label: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct PhraseProfile {
    pub id: u16,
    pub severity: u16,
    pub text: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct ZoneProfile {
    pub id: u16,
    pub county: u16,
    pub flood_plain: bool,
    pub radio_channel: u16,
    pub label: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct FormProfile {
    pub id: u16,
    pub revision: u16,
    pub retention_days: u16,
    pub label: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct PlaybookProfile {
    pub id: u16,
    pub phase: u16,
    pub actions: u16,
    pub label: &'static str,
}

pub fn shelter_profile(index: usize) -> &'static ShelterProfile {
    shelters::SHELTERS
        .get(index % shelters::SHELTERS.len())
        .unwrap_or(&shelters::SHELTERS[0])
}

pub fn phrase(seed: u32) -> &'static str {
    phrases::PHRASES
        .get(seed as usize % phrases::PHRASES.len())
        .map(|p| p.text)
        .unwrap_or("field report")
}

pub fn routing_hint(seed: u32) -> u32 {
    shelters::checksum_hint(seed)
        ^ supplies::checksum_hint(seed.rotate_left(3))
        ^ routes::checksum_hint(seed.rotate_left(7))
        ^ zones::checksum_hint(seed.rotate_left(11))
}
