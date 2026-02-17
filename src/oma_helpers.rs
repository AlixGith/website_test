use std::collections::HashMap;
pub use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs;
use std::io::BufRead;
use std::path::Path;


/*****************************************************************************
 *                      Arg parser
 *****************************************************************************/

use clap::Parser;

// TODO read config file also: https://stackoverflow.com/a/74070134

/// Simple Oma-Radio website builder
#[derive(Parser, Debug, Clone, Serialize)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// path of the oma-radio soundbase directory
    #[arg(long, env)]
    pub soundbase_path: String,

    /// hostname (or ip?) of radio head
    #[arg(long, env)]
    pub radio_host: String,

    /// Prefix of all website path in resulting documents
    #[arg(long, env, default_value_t=String::from(""))]
    pub website_prefix: String,

    /// Prefix of all soundbase path in resulting documents
    #[arg(long, env, default_value_t=String::from(""))]
    pub soundbase_prefix: String,

    /// Url of player js/css files
    #[arg(long, env, default_value_t=String::from("/player/"))]
    pub player_url: String,

    /// output of randered files
    #[arg(long, env)]
    pub output: String,

    /// Static directory to include
    #[arg(long, env, default_value_t=String::from("static"))]
    pub static_dir: String,

    /// main Prg file or link (simple name)
    #[arg(long, env, default_value_t=String::from("Programme-Defaut"))]
    pub prg_name: String,

    /// Manager url
    #[arg(long, env, default_value_t=String::from("/manager"))]
    pub manager_url: String,

    /// Websocket port
    #[arg(long, env, default_value_t=String::from("2000"))]
    pub websocket_port: String,

	/// Success message
	#[arg(long, env, default_value_t=String::from("OK. Terminé sans erreur blocante."))]
	pub success_message: String,

	///// Programation tables patterns
	//#[arg(long, env, value_delimiter='|', default_values_t=vec![String::from("_Grille-{day}_{hour}"), String::from("Playliste-{day}_{hour}")])]
	//pub programation_patterns: Vec<String>,

	///// Programation tables titles
	//#[arg(long, env, value_delimiter='|', default_values_t=vec![String::from("Grille de programmes"), String::from("Plages musicales")])]
	//pub programation_titles: Vec<String>,

	///// Programation days
	//#[arg(long, env, value_delimiter=',', default_values_t=vec![String::from("Lundi"),String::from("Mardi"),String::from("Mercredi"),String::from("Jeudi"),String::from("Vendredi"),String::from("Samedi"),String::from("Dimanche")])]
	//pub programation_days: Vec<String>,

	///// Programation hours
	//#[arg(long, env, value_delimiter=',', default_values_t=vec![String::from("00"), String::from("01"), String::from("02"), String::from("03"), String::from("04"), String::from("05"), String::from("06"), String::from("07"), String::from("08"), String::from("09"), String::from("10"), String::from("11"), String::from("12"), String::from("13"), String::from("14"), String::from("15"), String::from("16"), String::from("17"), String::from("18"), String::from("19"), String::from("20"), String::from("21"), String::from("22"), String::from("23")])]
	//pub programation_hours: Vec<String>,


	///// Programation plages
	//#[arg(long, env, value_delimiter=',', default_values_t=vec![String::from("Matin"),String::from("Aprem"),String::from("Soir"),String::from("Nuit")])]
	//pub programation_plages: Vec<String>,
}




custom_error::custom_error! { pub CustomError
	/* Custom Io error to ship filename with it */
    Io {
		msg: String,
        path: PathBuf
    } = @{format!("ERREUR : {}. {}", msg, path.display())},

	DatabaseError {
		path: PathBuf,
		line: u64,
		msg: String
	} = @{format!("ERREUR : dans la base de son, {}. Fichier : {}:{}", msg, path.display(), line)},
}


/*****************************************************************************
 *                      Oma fiche
 *****************************************************************************/

pub type FicContent = HashMap<String, String>;

//pub struct Fic {
//    content: FicContent,
//    filename: String,
//    oma: Oma,
//}
//
//impl Fic {
//    pub fn new<T: Into<String>>(filename: T, content:FicContent,  oma: &Oma) -> Self {
//        Self {
//            content: content,
//            filename: filename.into(),
//            oma: oma.clone(),
//        }
//    }
//}


/*****************************************************************************
 *                      Oma central class
 *****************************************************************************/

// Oma is the website builder. TODO Rename it?
#[derive(Clone)]
#[derive(Serialize)]
#[derive(Debug)]
pub struct Oma {
    pub args: Args,
}

impl Oma {
    pub fn new () -> Self {
        Self {
            args: Args::parse(),
        }
    }

    //pub fn new_fic<T: Into<String>>(&self, filename: T) -> Fic {
    //    let f = filename.into();
    //    Fic::new(&f, self.load_fic(&f).expect("Error loading fic"), &self)
    //}

    pub fn new_serie<T: Into<String>>(&self, filename: T) -> Result<Serie, Box<dyn std::error::Error>> {
        let f = filename.into();
        let mut s = Serie::new(&self, f);
        s.read_from_soundbase()?;
        Ok(s)
    }

    /* Read file content into String */
    pub fn get_file_content<T: Into<String>> (&self, filename: T, typ: T) -> Result<String, Box<dyn std::error::Error>> {
        /* Compute fic path */
        let f = filename.into();
        if f == "Vide" || f.is_empty() {
            return Ok("".to_string())
        }
        let t = typ.into();
        let path = self.path_of(&f, &t);
        match fs::read_to_string(&path) {
            Ok(v) => Ok(v),
            Err(_) => Err(Box::new(CustomError::Io {
                msg: "impossible de lire la fiche vérifiez son nom exact".to_string(),
                path: path,
            }))
        }
    }

    /* Load fic into FicContent
	* Returns empty content if fic doesnt exist
	*/
    pub fn load_fic<T: Into<String>> (&self, filename: T, follow: bool, accept_percent: bool, allow_absent: bool, default_values: FicContent) -> Result<FicContent, Box<dyn std::error::Error>> {
        /* Compute fic path */
        let f = filename.into();
        let path = self.path_of(&f, "fic");

        let mut content = default_values;

        if path.exists() || ! allow_absent {
            /* Open file */
            let file = match fs::File::open(&path) {
                Ok(f) => f,
                Err(_) => return Err(Box::new(CustomError::Io {
                    msg: "la fiche na pas été trouvée. Vérifiez son nom".to_string(),
                    path: path,
                }))
            };

            /* Create reader */
            let reader = std::io::BufReader::new(file);

            //TODO use it?
            //let mut buffer = String::new();

            /* Read fic content */
            let mut i=1;
            for line in reader.lines() {
                let line = line?;
                if line.is_empty() { continue; }

                /* Split line in key : value */
                let split: Vec<&str> = line.splitn(2, " : ").collect();
                let key = split[0].to_string();

                /* Check if value is empty */
                let mut value;
                if split.len() == 1 {
                    return Err(Box::new(CustomError::DatabaseError {path: path, line: i, msg: "ligne de fiche malformée".to_string()}));
                } else {
                    value = split[1].to_string();
                    if value == "Vide" {
                        value = "".to_string();
                    }
                }


                /* If fic is a link, follow. Unless it has % */
                if key == "Reel" {
                    if value.contains("%") {
                        if ! accept_percent {
                            return Err(Box::new(CustomError::DatabaseError {path:path, line:i, msg:format!("un % a été trouvé dans un lien. Impossible de le résoudre : {}",value)}));
                        }
                    } else if follow && value != "Vide" && value != "vide" && ! value.is_empty() {
                        return self.load_fic(value, follow, accept_percent, allow_absent, FicContent::new());
                    } else {
                    	return Err(Box::new(CustomError::DatabaseError{path: path, line: i, msg:"un lien a été trouvé alors que l’on cherchait une fiche".to_string()}));
					}
                }

				/* If key is a tag, try to convert it to Titre or Auteur */
				if key.starts_with("Tag") && key != "TagTime" {
					let i_json:usize = value.find("{").unwrap_or(0);
					if i_json != 0 {
						let tag_data:Result<serde_json::Value, serde_json::Error> = serde_json::from_str(&value[i_json..value.len()-1]);

						if let Ok(tag_data) = &tag_data {
							let v_a = &tag_data["A"];
							if *v_a != serde_json::Value::Null {
								content.insert("Auteur".to_string(), v_a.to_string());
							}

							let v_t = &tag_data["T"];
							if *v_t != serde_json::Value::Null {
								content.insert("Titre".to_string(), v_t.to_string());
							}
						} else {
							println!("Erreur à la lecture du json dans la fiche {} ligne {}", f, i);
						}
					}
				}

                /* Insert key : value in FicContent */
               	content.insert(key, value);

                i+=1;
            }

			if ! content.is_empty() {
            	/* Read metadata */
            	let metadata = fs::metadata(path)?;

            	/* Add last modified time */
            	if content.get("LastModified") == None {

            	    if let Ok(time) = metadata.modified() {
            	        let secs = time.duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_secs();
            	        content.insert("LastModified".to_string(), secs.to_string());
            	    } else {
            	        println!("metadata.modified() not supported on this platforme");
            	    }
            	}

            	/* Add creation time */
            	if content.get("Creation") == None {

            	    if let Ok(time) = metadata.created() {
            	        let secs = time.duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_secs();
            	        content.insert("Creation".to_string(), secs.to_string());
            	    } else {
            	        println!("metadata.created() not supported on this platforme");
            	        content.insert("Creation".to_string(), "0".to_string());
            	    }
            	}
        		content.insert("_filename".to_string(), f.clone());
			}
        } else {
			return Ok(content)
		}

        Ok(content)
    }

    /* Return file path in soundbase */
    pub fn path_of (&self, simple_name: &str, typ: &str) -> PathBuf {
        let mut path = Path::new(&self.args.soundbase_path).join(typ).join(simple_name);
        path.set_extension(typ);
        return path;
    }
}

/*****************************************************************************
 *              Program data structure
 ****************************************************************************/
use std::str::FromStr;
use std::fmt;

#[derive(PartialEq, PartialOrd)]
enum PrgSynchro {
    Heure,
    Jour,
    Semaine,
    Mois,
}

impl FromStr for PrgSynchro {
    type Err = ();

    fn from_str(input: &str) -> Result<PrgSynchro, Self::Err> {
        match input {
            "Heure"   => Ok(PrgSynchro::Heure),
            "Jour"    => Ok(PrgSynchro::Jour),
            "Semaine" => Ok(PrgSynchro::Semaine),
            "Mois"    => Ok(PrgSynchro::Mois),
            _         => Err(()),
        }
    }
}

impl fmt::Display for PrgSynchro {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match self {
           PrgSynchro::Heure   => write!(f, "heure"),
           PrgSynchro::Jour    => write!(f, "jour"),
           PrgSynchro::Semaine => write!(f, "semaine"),
           PrgSynchro::Mois    => write!(f, "mois"),
       }
    }
}



//#[derive(Serialize, Debug, Clone)]
//pub struct PrgLine {
//    mode: String,
//    year: String,
//    month: String,
//    day: String,
//    hour: String,
//    minute: String,
//    second: String,
//    command: String,
//    arg1: String,
//    arg2: String,
//    comment: Vec<String>,
//}
//
//impl PrgLine {
//    pub fn new (content: &str) -> Self {
//        let separator = content.find("#").unwrap_or(content.len());
//        let mut iter = content[0..separator].split_whitespace();
//        Self {
//            mode:    iter.next().expect(&format!("Malformed prg line: {}", content)).to_string(),
//            year:    iter.next().expect(&format!("Malformed prg line: {}", content)).to_string(),
//            month:   iter.next().expect(&format!("Malformed prg line: {}", content)).to_string(),
//            day:     iter.next().expect(&format!("Malformed prg line: {}", content)).to_string(),
//            hour:    iter.next().expect(&format!("Malformed prg line: {}", content)).to_string(),
//            minute:  iter.next().expect(&format!("Malformed prg line: {}", content)).to_string(),
//            second:  iter.next().expect(&format!("Malformed prg line: {}", content)).to_string(),
//            command: iter.next().expect(&format!("Malformed prg line: {}", content)).to_string(),
//            arg1:    iter.next().expect(&format!("Malformed prg line: {}", content)).to_string(),
//            arg2:    iter.next().unwrap_or("").to_string(),
//            comment: content[std::cmp::min(separator+1, content.len())..content.len()].split(",").map(str::to_string).collect(),
//        }
//    }
//}


/*****************************************************************************
 *         Programmation by liens (a custom type of file link)
 ****************************************************************************/

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ProgPattern {
	pattern: String,
	title: String,
	website: bool,
	#[serde(default)]
	vars: Vec<String>,
	#[serde(default)]
	expanded: HashMap<String, HashMap<String, FicContent>>,
	name: String, // not used, why do we deny unknown fields ?
	types: Vec<String>, // not used, why do we deny unknown fields ?
	#[serde(default)]
	tip: String, // not used, why do we deny unknown fields ?

}

#[derive(Serialize, Deserialize, Debug)]
pub struct Prog {
	/* {"jour":["lundi", "mardi"...]} */
	variables: HashMap<String, Vec<String>>,
	patterns: Vec<ProgPattern>,
	#[serde(default)]
	pub is_empty: bool,
}

impl Prog {
    pub fn new () -> Self {
		Self {
			variables: HashMap::new(),
			patterns: Vec::new(),
			is_empty: true,
		}
	}
}

/*****************************************************************************
 *              Data contains sets of series, config, pages...
 ****************************************************************************/

#[derive(Serialize, Debug)]
pub struct Data {
    oma: Oma,
    pub series: HashMap<String, Serie>,
    series_by_last_episode: Vec<(String,String)>,
    /* serie_name, episode_name, date */
    pub episodes_by_date: Vec<(String, String, String)>,
    pub pages: HashMap<String, FicContent>,
    pub config: FicContent,
    //program_lines: HashMap<String, String>,
    //program_variables: FicContent,
    pub program_table: HashMap<String, HashMap<String, FicContent>>,
	pub prog: Prog,

    /* prg lines grouped by categories */
    //pub program_lines: HashMap<String, Vec<PrgLine>>,
}

impl Data {
    pub fn new (oma: &Oma) -> Self {
        Self {
            oma: oma.clone(),
            series: HashMap::new(),
            series_by_last_episode: Vec::new(),
            episodes_by_date: Vec::new(),
            pages: HashMap::new(),
            config: HashMap::from([
                                  ("LastEpisodesInHome".to_string(), "8".to_string()),
                                  ("LastSeriesInHome".to_string(),   "8".to_string()),
                                  ("MainColor".to_string(),          "#AAF".to_string()),
                                  ("BackgroundColor".to_string(),    "#222".to_string()),
                                  ("TextColor".to_string(),          "#DDD".to_string()),
                                 ("BorderColor".to_string(),         "#DDD".to_string()),
                                  ("ItemBackground".to_string(),     "#333".to_string()),
                                  ("PrettyName".to_string(),         std::env::var("RADIO_NAME_PRETTY").unwrap_or("Radio Sans Nom".to_string())),
                                 ]),

            //program_lines: HashMap::new(),
            //program_variables: FicContent::new(),
            program_table: HashMap::new(),
			prog: Prog::new(),
            //program_lines: HashMap::new(),
        }
    }

    pub fn read_from_soundbase (&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Chargement de l’index des séries");
        let fic = self.oma.load_fic("_series-_index", false, false, true, FicContent::new())?;
        println!("Chargement des episodes");
        for serie_name in fic.get("SeriePodcasts").unwrap_or(&"".to_string()).as_str().split(" ") {
            if serie_name.is_empty() || serie_name.to_lowercase() == "vide" { continue; }
            //print!("{},", serie_name);
            self.series.insert(serie_name.to_string(), self.oma.new_serie(serie_name)?);
        }
        println!("Chargement de la config");
        let _ = self.read_config()?;
        println!("Chargement des pages");
        let _ = self.read_pages().unwrap_or(());
		println!("Chargement de la programmation");
		if let Err(e) = self.read_prog() {
			println!("{}", e);
			println!("-> La configuration du programme n’a pas pu être lue. Le programme ne sera pas généré.");
		}

        println!("Indexation des séries");
        self.index_series();
        println!("Indexation des épisodes");
        self.index_episodes();
        Ok(())
    }

    fn index_series (&mut self) {
        for (serie_name, serie) in &self.series {
            if serie_name.is_empty() { continue; }
            self.series_by_last_episode.push((serie_name.to_string(), serie.last_episode_time.to_string()));
        }
        self.series_by_last_episode.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap());
    }

    fn index_episodes (&mut self) {
        for (serie_name, serie) in &self.series {
            for (episode_name, episode) in &serie.episodes {
                if episode_name.is_empty() { continue; }
                /* Remove duplicates */
                let mut found = false;
                for e in &self.episodes_by_date {
                    if e.1 == *episode_name { found=true; continue; }
                }
                if found { continue; }
                self.episodes_by_date.push((serie_name.to_string(), episode_name.to_string(), episode["Creation"].to_string()));
            }
        }
        self.episodes_by_date.sort_by(|x,y| y.2.partial_cmp(&x.2).unwrap());
    }

    fn read_pages (&mut self) -> Result<(), Box<dyn std::error::Error>> {
		let pages_index = self.oma.load_fic("_PagesStatiques-_index", true, false, false, FicContent::new())?;
		if let Some(pages_names) = pages_index.get("PagesStatiques") {
        	for page_name in pages_names.split(" ") {
        	    if page_name.is_empty() { continue; }
        	    let fic = self.oma.load_fic(page_name, true, false, false, FicContent::new());
				if let Ok(p) = fic {
					self.pages.insert(page_name.to_string(), p);
				} else if let Err(e) = fic {
					println!("{}", e);
					println!("-> La page {} n’a pas été générée", page_name);
				}
			}
        } else {
			println!("-> Aucune page renseignée");
		}
        Ok(())
    }

    fn read_config (&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.config = self.oma.load_fic("_website-_config", false, false, true, self.config.clone()).expect("Cant read _website-_config fic");
        Ok(())
    }

    
    fn read_prog (&mut self) -> Result<(), Box<dyn std::error::Error>> {
		/* Load prog fic content */
        let fic = self.oma.load_fic(&self.oma.args.prg_name, true, false, false, FicContent::new())?;
        let prog_line = fic.get("Prog");
		let Some(json) = prog_line else {
            return Err(Box::new(CustomError::DatabaseError {
				path: fic.get("_filename").unwrap().into(),
				line: 0,
				msg: "La ligne 'Prog' n’a pas été trouvée dans la fiche qui doit correspondre au programme".to_string()
			}));
		};
		self.prog = serde_json::from_str(json)?;
		self.prog.patterns.retain(|pattern| pattern.website);

		/* Var extraction regex */
		let re = regex::Regex::new(r"\{([a-zA-Z]*)\}").unwrap();

		for pattern in &mut self.prog.patterns {
			/* Get vars from pattern */
			pattern.vars = re.captures_iter(&pattern.pattern).map(|caps| {
				caps[1].to_string()
			}).collect();
		}

		/* Coherence verification */
		let prg_coherence_err = Err(Box::new(CustomError::DatabaseError {
		    path: fic.get("_filename").unwrap().into(),
		    line: 65565,
		    msg: "tous les patterns du site web doivent avoir les mêmes variables à l’exception de la dernière. Vérifiez votre configuration de programmation.".to_string(),
		}));
		let first_vars = &self.prog.patterns[0].vars;
		let mut first_loop = true;
		for pattern in &self.prog.patterns {
			if first_loop {
				first_loop = false;
			} else {
				if pattern.vars.len() != first_vars.len() {
					return prg_coherence_err?;
				/* If var arrays are differents (the last part can be) */
				} else if pattern.vars[..pattern.vars.len()-1] != first_vars[..first_vars.len()-1] {
					return prg_coherence_err?;
				}
			}
		}

		self.prog.is_empty = false;

		Ok(())	
	}


}


/*****************************************************************************
 *              Serie contain its description and episodes
 *****************************************************************************/
#[derive(Serialize)]
#[derive(Debug)]
pub struct Serie {
	//TODO ugly to clone oma everywhere. Try to multiply refs
    oma: Oma,
    filename: String,
    fic: FicContent,
    pub episodes: HashMap<String, FicContent>,
    last_episode_time: String,
}

impl Serie {
    pub fn new (oma: &Oma, filename: String) -> Self {
        Self {
            oma: oma.clone(),
            filename: filename,
            fic: FicContent::new(),
            episodes: HashMap::new(),
            last_episode_time: "0".to_string(),
        }
    }


    pub fn read_from_soundbase (&mut self) -> Result<(), Box<dyn std::error::Error>> {
        /* Load fic */
        self.fic = self.oma.load_fic(&self.filename, true, false, false, FicContent::new())?;

        /* Look for Episode key */
        if let Some(episodes) = self.fic.get("Episodes") {
            for episode_name in episodes.as_str().split(" ") {
                /* Load every episode fic */
                if episode_name.is_empty() || episode_name.to_lowercase() == "vide" { continue; }
                self.episodes.insert(episode_name.to_string(), self.oma.load_fic(episode_name, true, false, false, FicContent::new())?);
            }
        }
        self.fic.insert("EpisodeNumber".to_string(), self.episodes.len().to_string());

        /* Find the most recent one */
        self.compute_last_episode_time();
        Ok(())
    }


    /* Loop every episode to find the most recent one */
    fn compute_last_episode_time (&mut self) {
		// TODO better error message when missing Creation key
        for (_episode_name, fic) in &self.episodes {
            if fic["Creation"] > self.last_episode_time {
                self.last_episode_time = fic["Creation"].clone()
            }
        }
    }
}
