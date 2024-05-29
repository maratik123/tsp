use crate::types::field::section_code::{
    AirportSubsectionCode, AirspaceSubsectionCode, CompanyRoutesSubsectionCode,
    EnrichedSectionCode, EnrouteSubsectionCode, HeliportSubsectionCode, MoraSubsectionCode,
    NavaidSubsectionCode, SectionCode, TablesSubsectionCode,
};

// 5.4 Section Code
pub fn parse_section_code(section_code: u8) -> Option<SectionCode> {
    Some(match section_code {
        b'A' => SectionCode::Mora,
        b'D' => SectionCode::Navaid,
        b'E' => SectionCode::Enroute,
        b'H' => SectionCode::Heliport,
        b'P' => SectionCode::Airport,
        b'R' => SectionCode::CompanyRoutes,
        b'T' => SectionCode::Tables,
        b'U' => SectionCode::Airspace,
        _ => None?,
    })
}

// 5.5 Subsection Code
pub fn parse_subsection_code(
    section_code: SectionCode,
    subsection_code: u8,
) -> Option<EnrichedSectionCode> {
    Some(match section_code {
        SectionCode::Mora => {
            EnrichedSectionCode::Mora(parse_mora_subsection_code(subsection_code)?)
        }
        SectionCode::Navaid => {
            EnrichedSectionCode::Navaid(parse_navaid_subsection_code(subsection_code)?)
        }
        SectionCode::Enroute => {
            EnrichedSectionCode::Enroute(parse_enroute_subsection_code(subsection_code)?)
        }
        SectionCode::Heliport => {
            EnrichedSectionCode::Heliport(parse_heliport_subsection_code(subsection_code)?)
        }
        SectionCode::Airport => {
            EnrichedSectionCode::Airport(parse_airport_subsection_code(subsection_code)?)
        }
        SectionCode::CompanyRoutes => EnrichedSectionCode::CompanyRoutes(
            parse_company_routes_subsection_code(subsection_code)?,
        ),
        SectionCode::Tables => {
            EnrichedSectionCode::Tables(parse_tables_subsection_code(subsection_code)?)
        }
        SectionCode::Airspace => {
            EnrichedSectionCode::Airspace(parse_airspace_subsection_code(subsection_code)?)
        }
    })
}

fn parse_airspace_subsection_code(subsection_code: u8) -> Option<AirspaceSubsectionCode> {
    Some(match subsection_code {
        b'C' => AirspaceSubsectionCode::ControlledAirspace,
        b'F' => AirspaceSubsectionCode::FirUir,
        b'R' => AirspaceSubsectionCode::RestrictiveAirspace,
        _ => None?,
    })
}

fn parse_tables_subsection_code(subsection_code: u8) -> Option<TablesSubsectionCode> {
    Some(match subsection_code {
        b'C' => TablesSubsectionCode::CruisingTables,
        b'G' => TablesSubsectionCode::GeographicalReference,
        _ => None?,
    })
}

fn parse_company_routes_subsection_code(
    subsection_code: u8,
) -> Option<CompanyRoutesSubsectionCode> {
    Some(match subsection_code {
        b' ' => CompanyRoutesSubsectionCode::CompanyRoutes,
        b'A' => CompanyRoutesSubsectionCode::AlternateRecords,
        _ => None?,
    })
}

fn parse_airport_subsection_code(subsection_code: u8) -> Option<AirportSubsectionCode> {
    Some(match subsection_code {
        b'A' => AirportSubsectionCode::ReferencePoints,
        b'B' => AirportSubsectionCode::Gates,
        b'C' => AirportSubsectionCode::TerminalWaypoints,
        b'D' => AirportSubsectionCode::Sids,
        b'E' => AirportSubsectionCode::Stars,
        b'F' => AirportSubsectionCode::ApproachProcedures,
        b'G' => AirportSubsectionCode::Runways,
        b'I' => AirportSubsectionCode::LocalizerGlideSlope,
        b'K' => AirportSubsectionCode::Taa,
        b'L' => AirportSubsectionCode::Mls,
        b'M' => AirportSubsectionCode::LocalizerMarker,
        b'N' => AirportSubsectionCode::TerminalNdb,
        b'P' => AirportSubsectionCode::PathPoint,
        b'R' => AirportSubsectionCode::FltPlanningArrDep,
        b'S' => AirportSubsectionCode::Msa,
        b'T' => AirportSubsectionCode::GlsStation,
        b'V' => AirportSubsectionCode::Communications,
        _ => None?,
    })
}

fn parse_heliport_subsection_code(subsection_code: u8) -> Option<HeliportSubsectionCode> {
    Some(match subsection_code {
        b'A' => HeliportSubsectionCode::Pads,
        b'C' => HeliportSubsectionCode::TerminalWaypoints,
        b'D' => HeliportSubsectionCode::Sids,
        b'E' => HeliportSubsectionCode::Stars,
        b'F' => HeliportSubsectionCode::ApproachProcedures,
        b'K' => HeliportSubsectionCode::Taa,
        b'S' => HeliportSubsectionCode::Msa,
        b'V' => HeliportSubsectionCode::Communications,
        _ => None?,
    })
}

fn parse_enroute_subsection_code(subsection_code: u8) -> Option<EnrouteSubsectionCode> {
    Some(match subsection_code {
        b'A' => EnrouteSubsectionCode::Waypoints,
        b'M' => EnrouteSubsectionCode::AirwayMarkers,
        b'P' => EnrouteSubsectionCode::HoldingPatterns,
        b'R' => EnrouteSubsectionCode::AirwaysAndRoutes,
        b'T' => EnrouteSubsectionCode::PreferredRoutes,
        b'U' => EnrouteSubsectionCode::AirwayRestrictions,
        b'V' => EnrouteSubsectionCode::Communications,
        _ => None?,
    })
}

fn parse_navaid_subsection_code(subsection_code: u8) -> Option<NavaidSubsectionCode> {
    Some(match subsection_code {
        b' ' => NavaidSubsectionCode::VhfNavaid,
        b'B' => NavaidSubsectionCode::NdbNavaid,
        _ => None?,
    })
}

fn parse_mora_subsection_code(subsections_code: u8) -> Option<MoraSubsectionCode> {
    Some(match subsections_code {
        b'S' => MoraSubsectionCode::GridMora,
        _ => None?,
    })
}
