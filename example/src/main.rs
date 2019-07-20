use orm::Client;
use orm::TransactionalClient;

use postgres::{Connection, TlsMode};

pub mod models;

use models::*;

fn main() {
    env_logger::init();

    let conn =
        Connection::connect("postgres://scales@localhost:5432/scales", TlsMode::None).unwrap();

    let client = Client::new(conn);

    {
        let mut trans = client.transaction().unwrap();

        println!("inserting 10000 entries...");

        for _ in 0..10000 {
            trans
                .insert(Scale {
                    id: 0,
                    serial: "1234567890".to_string(),
                })
                .unwrap();
        }

        trans.commit().unwrap();

        println!("done");
    }

    //client.query::<Scale>("@select", Scale::COLS::<);

    //client.query::<Scale>().where(Scale::COLS::id < "drölf");
    //let scales: Vec<Scale> = client.query().where("scale.id = $1", &["drölf"]).join_where(Scale::COLS::Measurement, "date > $2");

    // 'SELECT scale.* FROM scale INNER JOIN measurement ON [...] WHERE [...]'

    /*
    scale.id | scale.name | measurement.date | measurement.data
           1 |    MioMate |       2019-05-01 |            [...]
           1 |    MioMate |       2019-05-01 |            [...]
           1 |    MioMate |       2019-05-02 |            [...]
           1 |    MioMate |       2019-05-02 |            [...]
           2 |    MioCola |       2019-05-01 |            [...]
    */

    //client.from(Scale::TABLE).select_all()

    // trans.commit().unwrap();

    // client.insert(Scale {
    //     id: 0,
    //     serial: "hello".to_string(),
    // }).unwrap();

    // println!("done.");
    // trans.insert(Measurement {
    //     id: 0,
    //     raw_value: 2.5,
    //     scale: ToOne {
    //         id: 1,
    //         entity: None
    //     }
    // });
    //     let scales: Vec<Scale> = client.query("@select", &[]);
    //     // let measurements: Vec<Measurement> = client.query("@select", &[]);
    //     for e in scales {
    //         println!("{:?}", e);
    //     //     print!(".");
    //     }
}
