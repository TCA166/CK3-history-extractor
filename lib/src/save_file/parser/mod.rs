/// Commonly used types and other abstractions within the parser
pub mod types;

/// A submodule that provides the parser output objects.
/// The parser uses [GameObject](crate::parser::game_object::SaveFileObject) to
/// store the parsed data and structures in [structures](crate::structures) are
/// initialized from these objects. This is our workaround for the lack of
/// reflection in Rust, and it puts one more layer of abstraction between the
/// parser and the structures. Jomini style would be to have the structures
/// directly initialized from the token tape, but that wouldn't play well with
/// the way we store everything in a central [GameState] object.
mod game_object;
pub use game_object::{
    ConversionError, GameObjectCollection, GameObjectMap, GameObjectMapping, KeyError,
    SaveFileObject, SaveFileValue, SaveObjectError,
};

/// A submodule that provides the [Section] object, which allows the user to
/// choose which sections should be parsed.
mod section;
pub use section::{BinarySection, SaveFileSection, SectionError, TextSection};

/// A submodule that provides the [yield_section] function, which is used to
/// iterate over the save file and return the next section.
mod section_reader;
pub use section_reader::{BinarySectionReader, TextSectionReader};

mod error;
pub use error::ParsingError;

#[cfg(test)]
mod tests {

    use jomini::{self, text::TokenReader};

    use super::*;

    #[test]
    fn test_save_file() -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = TextSectionReader::new(TokenReader::from_slice(
            b"
        test={
            test2={
                test3=1
            }
        }
        ",
        ));
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        let object = section.parse().unwrap();
        let test2 = object
            .as_map()?
            .get("test2")
            .unwrap()
            .as_object()?
            .as_map()?;
        assert_eq!(test2.get("test3").unwrap().as_integer()?, 1);
        return Ok(());
    }

    #[test]
    fn test_save_file_array() -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = TextSectionReader::new(TokenReader::from_slice(
            b"
            test={
                test2={
                    1
                    2
                    3
                }
                test3={ 1 2 3}
            }
        ",
        ));
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        let object = section.parse().unwrap();
        let test2 = object.as_map()?.get("test2").unwrap().as_object()?;
        let test2_val = test2.as_array()?;
        assert_eq!(test2_val.get_index(0)?.as_integer()?, 1);
        assert_eq!(test2_val.get_index(1)?.as_integer()?, 2);
        assert_eq!(test2_val.get_index(2)?.as_integer()?, 3);
        let test3 = object.as_map()?.get("test3").unwrap().as_object()?;
        let test3_val = test3.as_array()?;
        assert_eq!(test3_val.get_index(0)?.as_integer()?, 1);
        assert_eq!(test3_val.get_index(1)?.as_integer()?, 2);
        assert_eq!(test3_val.get_index(2)?.as_integer()?, 3);
        Ok(())
    }

    #[test]
    fn test_weird_syntax() -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = TextSectionReader::new(TokenReader::from_slice(
            b"
        test={
            test2={1=2
                3=4}
            test3={1 2
                3}
            test4={1 2 3}
            test5=42
        }
    ",
        ));
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        let object = section.parse().unwrap();
        let test2 = object
            .as_map()?
            .get("test2")
            .unwrap()
            .as_object()?
            .as_map()?;
        assert_eq!(test2.get("1").unwrap().as_integer()?, 2);
        assert_eq!(test2.get("3").unwrap().as_integer()?, 4);
        Ok(())
    }

    #[test]
    fn test_array_syntax() -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = TextSectionReader::new(TokenReader::from_slice(
            b"
            test={
                test2={ 1 2 3 }
            }
        ",
        ));
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        let object = section.parse().unwrap();
        let test2 = object
            .as_map()?
            .get("test2")
            .unwrap()
            .as_object()?
            .as_array()?;
        assert_eq!(test2.get_index(0)?.as_integer()?, 1);
        assert_eq!(test2.get_index(1)?.as_integer()?, 2);
        assert_eq!(test2.get_index(2)?.as_integer()?, 3);
        assert_eq!(test2.len(), 3);
        Ok(())
    }

    #[test]
    fn test_unnamed_obj() -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = TextSectionReader::new(TokenReader::from_slice(
            b"
        3623={
            name=\"dynn_Sao\"
            variables={
                data={
                        {
                            flag=\"ai_random_harm_cooldown\"
                            tick=7818
                            data={
                                type=boolean
                                identity=1
                            }
                        }
                        {
                            something_else=\"test\"
                        }
                    }
                }
            }
        }
        ",
        ));
        let object = reader.next().unwrap().unwrap().parse().unwrap();
        let variables = object
            .as_map()?
            .get("variables")
            .unwrap()
            .as_object()?
            .as_map()?;
        let data = variables.get("data").unwrap().as_object()?.as_array()?;
        assert_ne!(data.len(), 0);
        Ok(())
    }

    #[test]
    fn test_example_1() -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = TextSectionReader::new(TokenReader::from_slice(b"
        3623={
            name=\"dynn_Sao\"
            variables={
                data={ {
                        flag=\"ai_random_harm_cooldown\"
                        tick=7818
                        data={
                            type=boolean
                            identity=1
                        }

                    }
         }
            }
            found_date=750.1.1
            head_of_house=83939093
            dynasty=3623
            historical={ 4440 5398 6726 10021 33554966 50385988 77977 33583389 50381158 50425637 16880568 83939093 }
            motto={
                key=\"motto_with_x_I_seek_y\"
                variables={ {
                        key=\"1\"
                        value=\"motto_the_sword_word\"
                    }
         {
                        key=\"2\"
                        value=\"motto_bravery\"
                    }
         }
            }
            artifact_claims={ 83888519 }
        }"));
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "3623");
        let object = section.parse().unwrap();
        assert_eq!(
            *(object.as_map()?.get("name").unwrap().as_string()?),
            "dynn_Sao".to_string()
        );
        let historical = object
            .as_map()?
            .get("historical")
            .unwrap()
            .as_object()?
            .as_array()?;
        assert_eq!(historical.get_index(0)?.as_integer()?, 4440);
        assert_eq!(historical.get_index(1)?.as_integer()?, 5398);
        assert_eq!(historical.get_index(2)?.as_integer()?, 6726);
        assert_eq!(historical.get_index(3)?.as_integer()?, 10021);
        assert_eq!(historical.get_index(4)?.as_integer()?, 33554966);
        assert_eq!(historical.len(), 12);
        Ok(())
    }

    #[test]
    fn test_space() -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = TextSectionReader::new(TokenReader::from_slice(
            b"
        test = {
            test2 = {
                test3 = 1
            }
            test4 = { a b c}
        }
        ",
        ));
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        let object = section.parse().unwrap();
        let test2 = object
            .as_map()?
            .get("test2")
            .unwrap()
            .as_object()?
            .as_map()?;
        assert_eq!(test2.get("test3").unwrap().as_integer()?, 1);
        let test4 = object.as_map()?.get("test4").unwrap().as_object()?;
        let test4_val = test4.as_array()?;
        assert_eq!(*(test4_val.get_index(0)?.as_string()?), "a".to_string());
        assert_eq!(*(test4_val.get_index(1)?.as_string()?), "b".to_string());
        assert_eq!(*(test4_val.get_index(2)?.as_string()?), "c".to_string());
        Ok(())
    }

    #[test]
    fn test_landed() -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = TextSectionReader::new(TokenReader::from_slice(
            b"
        c_derby = {
            color = { 255 50 20 }

            cultural_names = {
                name_list_norwegian = cn_djuraby
                name_list_danish = cn_djuraby
                name_list_swedish = cn_djuraby
                name_list_norse = cn_djuraby
            }

            b_derby = {
                province = 1621

                color = { 255 89 89 }

                cultural_names = {
                    name_list_norwegian = cn_djuraby
                    name_list_danish = cn_djuraby
                    name_list_swedish = cn_djuraby
                    name_list_norse = cn_djuraby
                }
            }
            b_chesterfield = {
                province = 1622

                color = { 255 50 20 }
            }
            b_castleton = {
                province = 1623

                color = { 255 50 20 }
            }
        }
        ",
        ));
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "c_derby");
        let object = section.parse().unwrap();
        let b_derby = object
            .as_map()?
            .get("b_derby")
            .unwrap()
            .as_object()?
            .as_map()?;
        assert_eq!(b_derby.get("province").unwrap().as_integer()?, 1621);
        let b_chesterfield = object
            .as_map()?
            .get("b_chesterfield")
            .unwrap()
            .as_object()?
            .as_map()?;
        assert_eq!(b_chesterfield.get("province").unwrap().as_integer()?, 1622);
        let b_castleton = object
            .as_map()?
            .get("b_castleton")
            .unwrap()
            .as_object()?
            .as_map()?;
        assert_eq!(b_castleton.get("province").unwrap().as_integer()?, 1623);
        Ok(())
    }

    #[test]
    fn test_invalid_line() -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = TextSectionReader::new(TokenReader::from_slice(
            b"
            nonsense=idk
            test={
                test2={
                    test3=1
                }
            }
        ",
        ));
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        let object = section.parse().unwrap();
        let test2 = object
            .as_map()?
            .get("test2")
            .unwrap()
            .as_object()?
            .as_map()?;
        assert_eq!(test2.get("test3").unwrap().as_integer()?, 1);
        Ok(())
    }

    #[test]
    fn test_empty() {
        let mut reader = TextSectionReader::new(TokenReader::from_slice(
            b"
            test={
            }
        ",
        ));
        let object = reader.next().unwrap().unwrap();
        assert_eq!(object.get_name(), "test");
    }

    #[test]
    fn test_arr_index() {
        let mut reader = TextSectionReader::new(TokenReader::from_slice(
            b"
            duration={ 2 0=7548 1=2096 }
        ",
        ));
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "duration");
        let object = section.parse().unwrap();
        let arr = object.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_id().unwrap(), 7548);
    }

    #[test]
    fn test_multi_key() -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = TextSectionReader::new(TokenReader::from_slice(
            b"
        test={
            a=hello
            a=world
        }
        ",
        ));
        let object = reader.next().unwrap().unwrap().parse().unwrap();
        let arr = object.as_map()?.get("a").unwrap().as_object()?.as_array()?;
        assert_eq!(arr.len(), 2);
        Ok(())
    }

    /*
    #[test]
    fn test_invalid_syntax_1() {
        let mut tape = get_test_obj(
            "
        test={
            a=hello
            b
        }
        ",
        )
        .unwrap();
        let object = yield_section(&mut tape).unwrap();
        assert!(object.unwrap().parse().is_ok())
    }

    #[test]
    fn test_invalid_syntax_2() {
        let mut tape = get_test_obj(
            "
        test={
            b
            a=hello
        }
        ",
        )
        .unwrap();
        let object = yield_section(&mut tape).unwrap();
        assert!(object.unwrap().parse().is_err())
    }
    */
}
