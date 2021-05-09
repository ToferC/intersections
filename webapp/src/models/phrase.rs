use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::{QueryDsl, BelongingToDsl};

use error_handler::error_handler::CustomError;
use crate::schema::phrases;
use database;

use crate::models::RawExperience;
use libretranslate::{translate, Language};

/* Query suggestion
    experience::tables.inner_join(phrases::table
        .on(experience::node_name.eq(phrases::id)
        .and(phrases::language.eq("foo"))
        .select((all, the, columns))
*/

#[derive(Debug, Serialize, Deserialize, Clone, AsChangeset, Insertable)]
#[table_name = "phrases"]
pub struct InsertablePhrase {
    pub lang: String,
    pub text: String,
}

impl InsertablePhrase {
    pub fn new(lang: &str, text: String) -> Self {
        InsertablePhrase {
            lang: lang.to_string(),
            text,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Associations, Insertable)]
#[table_name = "phrases"]
pub struct Phrases {
    pub id: i32,
    pub lang: String,
    pub text: String,
}

impl Phrases {
    
    pub fn create(phrase: &InsertablePhrase) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let p = diesel::insert_into(phrases::table)
            .values(phrase)
            .get_result(&conn)?;
        Ok(p)
    }

    pub fn add_translation(phrase: Phrases) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let p = diesel::insert_into(phrases::table)
            .values(phrase)
            .get_result(&conn)?;
        Ok(p)
    }

    pub fn update(id: i32, phrase: &InsertablePhrase) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let p = diesel::update(phrases::table)
            .filter(phrases::id.eq(id))
            .set(phrase)
            .get_result(&conn)?;
        Ok(p)
    }

    pub fn get_phrases_from_ids(ids: Vec<i32>, lang: &str) -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;
        let phrases = phrases::table
            .filter(phrases::id.eq_any(ids)
            .and(phrases::lang.eq(lang)))
            .load::<Phrases>(&conn)?;

            Ok(phrases)
    }

    pub fn find(id: i32, lang: &str) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let phrase = phrases::table.filter(phrases::id.eq(id)
            .and(phrases::lang.eq(lang)))
            .first(&conn)?;

        Ok(phrase)
    }

    pub fn delete(id: i32) -> Result<usize, CustomError> {
        let conn = database::connection()?;
        let res = diesel::delete(phrases::table.filter(phrases::id.eq(id))).execute(&conn)?;
        Ok(res)
    }
}

pub async fn generate_experience_phrases(lang: &str, ex: RawExperience) -> Result<Vec<i32>, CustomError> {
    // Translates a complete experience including node name and statements
    // Returns a String that is meant to be split on "\n."

    let mut translate_strings: Vec<String> = Vec::new();

    translate_strings.push(ex.node_name.clone());
    
    for s in &ex.statements {
        translate_strings.push(format!("{}.\n", &s));
    };

    let mut source = Language::English;
    let mut target = Language::French;

    let translate_lang = match &lang {
        &"en" => {
            "fr".to_string()
        },
        &"fr" => {
            source = Language::French;
            target = Language::English;
            "en".to_string()
        },
        _ => {
            "fr".to_string()
        },
    };

    println!("Translating experience: {}", &ex.node_name);

    let source = Language::English;

    let input = translate_strings.concat();

    let data = translate(source, target, input)
        .await
        .unwrap();

    let input = data.input.split(".\n");
    let output = data.output.split(".\n");

    let mut phrase_ids = Vec::new();

    for (i, o) in input.into_iter().zip(output) {

        let phrase = InsertablePhrase::new(lang, i.to_lowercase().trim().replace("/",""));

        let phrase = Phrases::create(&phrase).expect("Unable to create phrase");

        let trans = Phrases {
            id: phrase.id,
            lang: translate_lang.to_owned(),
            text: o.to_lowercase().trim().replace("/",""),
        };

        let translation = Phrases::add_translation(trans).expect("Unable to add translation phrase");

        println!("Success: {} ({}) -> {} ({})", &ex.node_name, phrase.id, &translation.text, translation.id);
        
        phrase_ids.push(phrase.id);
    };

    Ok(phrase_ids)
}
