use tsp::parser::record::parse_airport_primary_records;

fn main() {
    let record = b"SUSAP KSEAK1ASEA     0     \
        119YHN47265960W122184240E016000432         1800018000C    \
        MNAR    SEATTLE-TACOMA INTL           065001807";
    println!("{:#?}", parse_airport_primary_records(&record[..]).unwrap());
}
