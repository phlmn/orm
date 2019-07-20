use orm_derive::Entity;

#[derive(Debug, Entity)]
pub struct Scale {
    #[orm(generated, primary_key)]
    pub id: i64,

    pub serial: String,
    // #[orm(oneToMany(Measurement)]
    // pub measurements: ToMany<i64, Measurement>,
}

#[derive(Debug, Entity)]
pub struct Measurement {
    #[orm(generated, primary_key)]
    pub id: i64,

    pub raw_value: f32,
    //#[orm(manyToOne(Scale))]
    //pub scale: ToOne<i64, Scale>,
}
