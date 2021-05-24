use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::{QueryDsl};
use std::{collections::BTreeMap, sync::Arc};
use deepl_api::{DeepL, TranslatableTextList};

use error_handler::error_handler::CustomError;
use crate::schema::phrases;
use database;

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
    pub machine_translation: bool,
}

impl InsertablePhrase {
    pub fn new(lang: &str, text: String, machine_translation: bool) -> Self {
        InsertablePhrase {
            lang: lang.to_string(),
            text,
            machine_translation,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Associations, Insertable)]
#[table_name = "phrases"]
pub struct Phrases {
    pub id: i32,
    pub lang: String,
    pub text: String,
    pub machine_translation: bool,
}

impl Phrases {
    
    pub fn create(phrase: &InsertablePhrase) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let p = diesel::insert_into(phrases::table)
            .values(phrase)
            .get_result(&conn)?;
        Ok(p)
    }

    pub fn update_or_create(phrase: &InsertablePhrase) -> Result<Self, CustomError> {
        let conn = database::connection()?;

        let p = diesel::insert_into(phrases::table)
            .values(phrase)
            .on_conflict((phrases::id, phrases::lang))
            .do_update()
            .set(phrase)
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

    pub fn get_phrase_map(ids: Vec<i32>, lang: &str) -> Result<BTreeMap<i32, String>, CustomError> {
        let conn = database::connection()?;
        let phrases = phrases::table
            .filter(phrases::id.eq_any(ids)
            .and(phrases::lang.eq(lang)))
            .load::<Phrases>(&conn)?;

        let mut treemap = BTreeMap::new();

        for p in phrases {
            treemap.insert(p.id, p.text);
        };

        Ok(treemap)
    }

    pub fn find_from_text(text: &str, lang: &str) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let phrases = phrases::table
            .filter(phrases::text.eq(text)
            .and(phrases::lang.eq(lang)))
            .first(&conn)?;

            Ok(phrases)
    }

    pub fn find(id: i32, lang: &str) -> Result<Self, CustomError> {
        let conn = database::connection()?;
        let phrase = phrases::table.filter(phrases::id.eq(id)
            .and(phrases::lang.eq(lang)))
            .first(&conn)?;

        Ok(phrase)
    }

    pub fn find_all() -> Result<Vec<Self>, CustomError> {
        let conn = database::connection()?;
        let phrases = phrases::table.load::<Phrases>(&conn)?;

        Ok(phrases)
    }

    pub fn delete(id: i32) -> Result<usize, CustomError> {
        let conn = database::connection()?;
        let res = diesel::delete(phrases::table.filter(phrases::id.eq(id))).execute(&conn)?;
        Ok(res)
    }
}

pub async fn generate_phrases(texts: Vec<String>, lang: &str) -> Result<Vec<(i32, String)>, CustomError> {
    // Saves experience phrases in language of origin
    // Note that this also generates the node name for any new nodes

    let mut tree_map: Vec<(i32, String)> = Vec::new();

    for t in texts {
        let prep_phrase = InsertablePhrase::new(lang, t.to_lowercase().trim().replace("/",""), false);

        // check to see if phrase already exists, return phrase if it does
        let p = Phrases::find_from_text(&prep_phrase.text, &prep_phrase.lang);

        println!("Checking to see if phrase: {} exists", &prep_phrase.text);
        let phrase = match p {
            Ok(p) => {
                println!("Phrase \"{}\" Exists", &p.text);
                // return found phrase
                p
            },
            Err(e) => {
                println!("Does not exist - creating: {}", e);
                // add new phrase to db
                let p = Phrases::create(&prep_phrase).expect("Unable to create phrase");
                p
            }
        };

        // set raw_exp name_id. This will become the node name and generate the node slug
        tree_map.push((phrase.id, t));
    }
    
    Ok(tree_map)
}

pub async fn translate_phrases<'a>(phrase_vec: Arc<Vec<(i32, String)>>, lang: Arc<String>) -> Result<(), CustomError> {
    // Translates a complete experience including node name and statements
    // Returns a String that is meant to be split on "\n."

    let key = match std::env::var("DEEPL_API_KEY") {
        Ok(val) if val.len() > 0 => val,
        _ => {
            eprintln!("Error: no DEEPL_API_KEY found. Please provide your API key in this environment variable.");
            std::process::exit(1);
        }
    };

    let deepl = DeepL::new(key); 
    
    let mut source = "EN".to_string();
    let mut target = "FR".to_string();

    let lang = &*lang.clone();
    
    let translate_lang = match lang.as_str() {
        "en" => {
            "fr".to_string()
        },
        "fr" => {
            source = "FR".to_string();
            target = "EN".to_string();
            "en".to_string()
        },
        _ => {
            "fr".to_string()
        },
    };

    let texts: Vec<String> = phrase_vec.iter().map(|p| p.1.to_string()).collect();
    
    println!("Translating texts: {:?}", &*texts);
            
     // Translate Text
     let texts = TranslatableTextList {
        source_language: Some(source),
        target_language: target,
        texts: texts,
    };

    let translated = deepl.translate(None, texts).await.unwrap();
    
    for (input, translation) in phrase_vec.iter().zip(translated) {
        
        let trans = Phrases {
            id: input.0,
            lang: translate_lang.to_owned(),
            text: translation.text.trim().to_lowercase().replace("/",""),
            machine_translation: true,
        };
        
        // see if translation exists -- think this through
        let p = Phrases::find(trans.id, &trans.lang);
        
        println!("Checking to see if phrase: {} exists", &trans.text);
        let translation = match p {
            Ok(p) => {
                println!("Translation \"{}\" exists", &p.text);
                p
            },
            Err(e) => {
                println!("Does not exist - creating translation: {}", e);
                Phrases::add_translation(trans)?
            }
        };
        println!("Success - Name: {} ({}) -> {} ({})", &input.1, input.0, &translation.text, translation.id);
    }

    Ok(())
    
}


