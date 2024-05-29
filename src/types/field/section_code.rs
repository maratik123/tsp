#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SectionCode {
    Mora,
    Navaid,
    Enroute,
    Heliport,
    Airport,
    CompanyRoutes,
    Tables,
    Airspace,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EnrichedSectionCode {
    Mora(MoraSubsectionCode),
    Navaid(NavaidSubsectionCode),
    Enroute(EnrouteSubsectionCode),
    Heliport(HeliportSubsectionCode),
    Airport(AirportSubsectionCode),
    CompanyRoutes(CompanyRoutesSubsectionCode),
    Tables(TablesSubsectionCode),
    Airspace(AirspaceSubsectionCode),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MoraSubsectionCode {
    GridMora,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NavaidSubsectionCode {
    VhfNavaid,
    NdbNavaid,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EnrouteSubsectionCode {
    Waypoints,
    AirwayMarkers,
    HoldingPatterns,
    AirwaysAndRoutes,
    PreferredRoutes,
    AirwayRestrictions,
    Communications,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HeliportSubsectionCode {
    Pads,
    TerminalWaypoints,
    Sids,
    Stars,
    ApproachProcedures,
    Taa,
    Msa,
    Communications,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AirportSubsectionCode {
    ReferencePoints,
    Gates,
    TerminalWaypoints,
    Sids,
    Stars,
    ApproachProcedures,
    Runways,
    LocalizerGlideSlope,
    Taa,
    Mls,
    LocalizerMarker,
    TerminalNdb,
    PathPoint,
    FltPlanningArrDep,
    Msa,
    GlsStation,
    Communications,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CompanyRoutesSubsectionCode {
    CompanyRoutes,
    AlternateRecords,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TablesSubsectionCode {
    CruisingTables,
    GeographicalReference,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AirspaceSubsectionCode {
    ControlledAirspace,
    FirUir,
    RestrictiveAirspace,
}
