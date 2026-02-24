
/* Oma tool */
mod oma_helpers;
use oma_helpers::{Data, Oma, FicContent};

/* Tera loading */
use tera::{Tera, Value, Error, to_value};


/*For the news documents */
use std::fs;
use std::path::Path;

use include_dir::{include_dir, Dir};
	
/* Helper functions for templates */
use std::collections::HashMap;
fn helper_test(arg: &HashMap<String, Value>) -> Result<Value, Error> {
    Ok(to_value(arg["name"].as_str())?)
}

use chrono::NaiveDate;
fn date_to_ts(value: &Value, _arg: &HashMap<String, Value>) -> Result<Value, Error> {
	let d = NaiveDate::parse_from_str(value.as_str().unwrap(), "%Y-%m-%d").unwrap_or(NaiveDate::from_ymd_opt(1970, 01, 02).unwrap());
	Ok(to_value(d.format_localized("%d %B %Y", chrono::Locale::fr_FR).to_string())?)
}

fn tera_load_fic(oma: Oma) -> impl tera::Function {
	Box::new(move |args: &HashMap<String, Value>| -> Result<Value, Error> {
        match args.get("name") {
            Some(val) => match tera::from_value::<String>(val.clone()) {
				Ok(value) => Ok(tera::to_value(oma.load_fic(&value, true, false, true, FicContent::new()).expect(format!("Erreur à la récupération de la fiche {}\n", &value).as_str()))?),
                Err(_) => Err("oops".into()),
            },
            None => Err("name is required in tera_load_fic".into()),
		}
	})
}

/* Tera function to load txt content */
//XXX make it xss safe
fn txt_content(oma: Oma) -> impl tera::Function {
    Box::new(move |args: &HashMap<String, Value>| -> Result<Value, Error> {
        match args.get("name") {
            Some(val) => match tera::from_value::<String>(val.clone()) {
                Ok(v) =>  Ok(tera::to_value(oma.get_file_content(&v, &"txt".to_string()).expect(format!("ERREUR à la récupération du texte : {}\n", &v).as_str()))?),
                Err(_) => Err("oops".into()),
            },
            None => Err("name is required in txt_content".into()),
        }
    })
}


/* Main */

fn main() -> Result<(), Box<dyn std::error::Error>> {

	let mut detected_error = false;

    println!("Début de la génération du site web…");
    let oma = Oma::new();

    println!("Chargement des données depuis la base de son…");
    let mut data = Data::new(&oma);
    data.read_from_soundbase()?;

	println!("Création des répertoires…");
    let serie_dir = oma.args.output.clone() + "/serie/";
    let episode_dir = oma.args.output.clone() + "/episode/";
    std::fs::create_dir_all(serie_dir.clone())?;
    std::fs::create_dir_all(episode_dir.clone())?;
    let img_dir = oma.args.output.clone() + "/img/";
    std::fs::create_dir_all(img_dir.clone())?;
    std::fs::create_dir_all(oma.args.output.clone() + "/script/")?;
    let page_dir = oma.args.output.clone() + "/page/";
    std::fs::create_dir_all(page_dir.clone())?;


    println!("Construction de Tera et chargement des modèles de page…");
    let mut tera = Tera::default();
    tera.register_function("helper_test", helper_test);
    tera.register_filter("date_to_ts", date_to_ts);
    tera.register_function("txt_content", txt_content(oma.clone()));
    tera.register_function("tera_load_fic", tera_load_fic(oma.clone()));
    tera.add_raw_templates(vec![
        ("macros.html", include_str!("templates/macros.html")),
        ("base.html", include_str!("templates/base.html")),
        ("index.html", include_str!("templates/index.html")),
        ("programme.html", include_str!("templates/programme.html")),
        ("serie_index.html", include_str!("templates/serie_index.html")),
        ("episode_index.html", include_str!("templates/episode_index.html")),
        ("page.html", include_str!("templates/page.html")),
        ("serie.html", include_str!("templates/serie.html")),
        ("episode.html", include_str!("templates/episode.html")),
        ("single_episode.html", include_str!("templates/single_episode.html")),
        ("single_serie.html", include_str!("templates/single_serie.html")),
        ("last_series.html", include_str!("templates/last_series.html")),
		("episodes_serie.html", include_str!("templates/episodes_serie.html")),
        ("last_episodes.html", include_str!("templates/last_episodes.html")),
        ("Cards_brochure.html", include_str!("templates/Cards_brochure.html")),

        ("style.css", include_str!("templates/style.css")),
        ("jquery.modal90f6.css", include_str!("wp-content/themes/lnei-wp-theme/resources/assets/vendor/jquery.modal90f6.css")),
        ("customf9d8.css", include_str!("wp-content/themes/lnei-wp-theme-child-nova/dist/customf9d8.css")),
        ("jquery-3.6.0.minf9df.js", include_str!("wp-content/themes/lnei-wp-theme/resources/assets/vendor/jquery-3.6.0.minf9df.js")),
		
		

		
		("img/download.svg", include_str!("templates/img/download.svg")),
        ("img/earth.svg", include_str!("templates/img/earth.svg")),
        ("img/home.svg", include_str!("templates/img/home.svg")),
        ("img/play.svg", include_str!("templates/img/play.svg")),
        ("img/plus.svg", include_str!("templates/img/plus.svg")),
        ("img/rss.svg", include_str!("templates/img/rss.svg")),
        ("img/share.svg", include_str!("templates/img/share.svg")),
    ])?;

    let mut context = tera::Context::new();
    context.insert("data", &data);

	let favicon_path = std::path::Path::new(&oma.args.soundbase_path).join("website/favicon.webp");
	let logo_config = data.config.get("Logo");
	if let Some(logo) = logo_config {
		println!("Ajout de l’icone pour navigateur : {}", logo);
		let path = oma.path_of(logo, "webpL");
		std::fs::copy(&path, favicon_path).unwrap_or_else(|_|
			panic!("ERREUR Logo non trouvé {:}", logo)
		);
	} else {
		println!("Pas de logo renseigné, suppression de l’icone pour navigateur.");
		if favicon_path.exists() {
			let _ = std::fs::remove_file(favicon_path);
		}
	}

    println!("Construction des pages web standard…");
    let _ = std::fs::write(
        oma.args.output.clone() + "/index.html",
        tera.render("index.html", &context).unwrap()
    );


	if ! data.prog.is_empty {
    	let _ = std::fs::write(
    	    oma.args.output.clone() + "/programme.html",
    	    tera.render("programme.html", &context).unwrap()
    	);
	}

    let _ = std::fs::write(
        oma.args.output.clone() + "/serie/index.html",
        tera.render("serie_index.html", &context).unwrap()
    );

    let _ = std::fs::write(
        oma.args.output.clone() + "/episode/index.html",
        tera.render("episode_index.html", &context).unwrap()
    );

    let _ = std::fs::write(
        oma.args.output.clone() + "/style.css",
        tera.render("style.css", &context).unwrap()
    );

	let _ = std::fs::write(
        oma.args.output.clone() + "/jquery.modal90f6.css",
        tera.render("jquery.modal90f6.css", &context).unwrap()
    );
	let _ = std::fs::write(
        oma.args.output.clone() + "/customf9d8.css",
        tera.render("customf9d8.css", &context).unwrap()
    );
	let _ = std::fs::write(
        oma.args.output.clone() + "/jquery-3.6.0.minf9df.js",
        tera.render("jquery-3.6.0.minf9df.js", &context).unwrap()
    );
	
	let _ = std::fs::write(
	    oma.args.output.clone() + "/mainca34.js",
	    include_str!("wp-content/themes/lnei-wp-theme-child-nova/dist/mainca34.js"),
	).unwrap();

	let _ = std::fs::write(
		oma.args.output.clone() + "/wp-json",
		include_dir!("src/wp-json"),

	).unwrap();
	
	let _ = std::fs::write(
		oma.args.output.clone() + "/wp-includes",
		include_dir!("src/wp-includes"),

	).unwrap();
	
	let _ = std::fs::write(
		oma.args.output.clone() + "/wp-content",
		include_dir!("src/wp-content"),

	).unwrap();

	
	let favicon = std::fs::read(
    "src/templates/img/cropped-favicon-4-t-384x384.png"
	).unwrap();
	
	let _ = std::fs::write(
	    img_dir.clone() + "cropped-favicon-4-t-384x384.png",
	    favicon
	);
	
	 let _ = std::fs::write(
        img_dir.clone() + "earth.svg",
        tera.render("img/earth.svg", &context).unwrap()
    );

    let _ = std::fs::write(
        img_dir.clone() + "home.svg",
        tera.render("img/home.svg", &context).unwrap()
    );

    let _ = std::fs::write(
        img_dir.clone() + "play.svg",
        tera.render("img/play.svg", &context).unwrap()
    );

    let _ = std::fs::write(
        img_dir.clone() + "plus.svg",
        tera.render("img/plus.svg", &context).unwrap()
    );

    let _ = std::fs::write(
        img_dir.clone() + "rss.svg",
        tera.render("img/rss.svg", &context).unwrap()
    );

    let _ = std::fs::write(
        img_dir.clone() + "share.svg",
        tera.render("img/share.svg", &context).unwrap()
    );

    let _ = std::fs::write(
        oma.args.output.clone() + "/script/main.js",
        include_str!("script/main.js")
    );


    println!("Construction des pages web personnalisées…");
    for (page_name, fic) in data.pages {
		if fic.get("Titre").is_none() {
			println!("ERREUR : La page {} n’a pas de Titre dans sa fiche", page_name);
			detected_error = true;
		} else {
        	context.insert("page", &fic);
        	let _ = std::fs::write(
        	    page_dir.clone() + &page_name + ".html",
        	    tera.render("page.html", &context).unwrap()
        	);
		}
    }

    println!("Constructions des pages des séries et épisodes…");
    for (serie_name, serie) in data.series {
        context.insert("serie", &serie);
        context.insert("serie_name", &serie_name);
        let _ = std::fs::write(
            serie_dir.clone() + &serie_name + ".html",
            tera.render("serie.html", &context).unwrap()
        );

        /* Render episode page */
        //println!("Render {} episodes", serie.episodes.len());
        for (episode_name, fic) in serie.episodes {
            context.insert("episode", &fic);
            context.insert("episode_name", &episode_name);
            let _ = std::fs::write(
                episode_dir.clone() + &episode_name + ".html",
                tera.render("episode.html", &context).unwrap()
            );
        }
    }


	if detected_error {
		println!("Génération terminée, une ou plusieurs erreur(s) détectée(s).");
	} else {
		println!("{}", oma.args.success_message);
	}

    Ok(())
}
