use bytes::BytesMut;
use postgres::types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::error::Error;
use std::fmt;
use std::ops::Deref;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Point(postgis::ewkb::Point);

impl Point {
    pub fn new(x: f64, y: f64, srid: Option<i32>) -> Self {
        Point(postgis::ewkb::Point::new(x, y, srid))
    }
}

impl Deref for Point {
    type Target = postgis::ewkb::Point;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> FromSql<'a> for Point {
    fn from_sql(ty: &Type, raw: &[u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        let point: Result<postgis::ewkb::Point, Box<dyn Error + Sync + Send>> =
            FromSql::from_sql(ty, raw);
        point.map(|point| Point(point))
    }

    fn accepts(ty: &Type) -> bool {
        <postgis::ewkb::Point as FromSql>::accepts(ty)
    }
}

impl ToSql for Point {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        ToSql::to_sql(&self.0, ty, out)
    }

    to_sql_checked!();

    fn accepts(ty: &Type) -> bool {
        <postgis::ewkb::Point as ToSql>::accepts(ty)
    }
}

impl Serialize for Point {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Point", 3)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.serialize_field("srid", &self.srid)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Point {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            X,
            Y,
            Srid,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`x` or `y` or `srid`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "x" => Ok(Field::X),
                            "y" => Ok(Field::Y),
                            "srid" => Ok(Field::Srid),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct PointVisitor;

        impl<'de> Visitor<'de> for PointVisitor {
            type Value = Point;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Point")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Point, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let x = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let y = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let srid = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                Ok(Point::new(x, y, srid))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Point, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut x = None;
                let mut y = None;
                let mut srid = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::X => {
                            if x.is_some() {
                                return Err(de::Error::duplicate_field("x"));
                            }
                            x = Some(map.next_value()?);
                        }
                        Field::Y => {
                            if y.is_some() {
                                return Err(de::Error::duplicate_field("y"));
                            }
                            y = Some(map.next_value()?);
                        }
                        Field::Srid => {
                            if srid.is_some() {
                                return Err(de::Error::duplicate_field("srid"));
                            }
                            srid = Some(map.next_value()?);
                        }
                    }
                }
                let x = x.ok_or_else(|| de::Error::missing_field("x"))?;
                let y = y.ok_or_else(|| de::Error::missing_field("y"))?;
                let srid = srid.ok_or_else(|| de::Error::missing_field("srid"))?;
                Ok(Point::new(x, y, srid))
            }
        }

        const FIELDS: &'static [&'static str] = &["x", "y", "srid"];
        deserializer.deserialize_struct("Point", FIELDS, PointVisitor)
    }
}

#[test]
fn test_ser_de() {
    let point = Point::new(1.1234, -5.6789, None);
    let s = serde_json::to_string(&point).unwrap();
    let d: Point = serde_json::from_str(&s).unwrap();
    assert_eq!(d, point);
}
