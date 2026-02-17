/* Global variable holding last url browsed */
var OMA_LAST_URL = window.location.href;

/* Share episode function */
function openShare (url) {
	navigator.clipboard.writeText(url);
	alert('L’adresse a été copiée dans le presse-papier');
}

/* Program tabs function */
function openTab(index=0, value=null, auto=false) {
	/* Get tabs from url options and check for validity */
	var current_tabs = []
	var url_params = new URLSearchParams(window.location.search);
	if (url_params != null) {
		current_tabs = url_params.get("tab")
		if (current_tabs != null) {
			current_tabs = current_tabs.split('-');
			if (current_tabs.length != OMA_PROGRAM_DEFAULT_TAB.length) {
				current_tabs = OMA_PROGRAM_DEFAULT_TAB
			}
		} else {
			current_tabs = OMA_PROGRAM_DEFAULT_TAB
		}
	} else {
		current_tabs = OMA_PROGRAM_DEFAULT_TAB
	}

	/* Set the new tab value */
	if (value) {
		current_tabs[index] = value;
	}

	/* Reset selected tab class */
	var x = document.getElementsByClassName("tablink");
	for (var i=0 ; i<x.length ; i++) {
		x[i].classList.remove('selectedTab');
	}

	/* Add selected tab class */
	for (var i=0; i<current_tabs.length; i++) {
		document.getElementById("link-"+i+"-"+current_tabs[i]).classList.add('selectedTab')
	}

	/* Hide content */
	var x = document.getElementsByClassName("tabcontent");
	for (var i = 0; i < x.length; i++) {
	  x[i].style.display = "none";
	}

	/* Show selected content */
	var class_name="tab-" + current_tabs.join('-');
	var contents = document.getElementsByClassName(class_name);
	for (var i=0; i< contents.length ; i++) {
		contents[i].style.display = "block";
	}

	/* Save the new value in url to enable sharing */
	//internalNavigation(window.location.toString().replace(/\?[^#]*/, "?tab=" + current_tabs.join('-')));
	if (!auto) {
		internalNavigation("?tab=" + current_tabs.join('-'));
	}
}


/* progressive website enhancement */
function enableAdvancedSite (firstTime=true) {
	/* Show/hide different website parts */
	var elements = document.getElementsByClassName('toshow');
	for (var i=0 ; i<elements.length ; i=i) {
		elements[i].style = 'display:initial !important';
  	    elements[i].classList.remove("toshow");
	}
	elements = document.getElementsByClassName('tohide');
	for (var i=0 ; i<elements.length ; i++) {
		elements[i].style.display = "none";
	}

	/* Register the current (classicaly loaded) page to the js history */
	if (firstTime) {
		history.replaceState({url:window.location.href}, '', window.location);
	}
}


/* Dynamic page loading by replacing main content */
function fetchAndLoad (url) {

	/* Do JS navigation */
	return fetch(url)
    .then(response => response.text() )
	.then(content => {
		const doc = (new DOMParser()).parseFromString(content, "text/html");
		const element = document.getElementById('maincontent');
		const main = doc.getElementById('maincontent')
		element.parentNode.replaceChild(main, element);
		document.title = doc.title

		enableAdvancedSite(false);
	})
	.catch(err => {
		// TODO mettre un chargement pendant le changement de page pour éviter les clics rapides
		console.error("Erreur de navigation, essayez la version simple du site web");
		console.error(err);
	})

}

/* 
 * This function can be called by a clic on a link or with popstate event
 * (hence the scrolls variables)
 * e can be a clic event on <a> or an Url string
 */
function internalNavigation (e, pushHistory=true, scrollX=null, scrollY=null) {
	var url = e;
	/* If function is called via clic event */
	if (e.currentTarget) {
		/* Test if player was loaded, or if it is an external link */
		if ( !window.advancedSite || !window.OMA_DATA || !window.OMA_DATA.player || !e.currentTarget.href.startsWith(window.location.origin)) {
			return true
		}
		url = e.currentTarget.href;
	}

	var samepage = OMA_LAST_URL.replace(/#.*/, '') === url.replace(/#.*/, '') || url.startsWith('?');

	/* Push item on history stack */
	if (pushHistory) {
		history.replaceState({url:window.location.href, scrollX:window.scrollX, scrollY:window.scrollY}, '', window.location.href)
		history.pushState({url: url, scrollX:window.scrollX, scrollY:window.scrollY}, '', url);
		//TODO check if last state is the same to avoid duplicated entries
	}
	
	/* Create or get promise */
	var promise = null;
	if (!samepage) {
		promise = fetchAndLoad(url);
	} else {
		promise = new Promise (resolve => {
			resolve();
		});
	}

	/* Wait for page loading then scroll to the right place */
	promise.then( () => {
		const hash = window.location.hash.substring(1);
		const target = hash !== '' ? document.getElementById(hash) : null;
		if (scrollX !== null && scrollY !== null) {
			window.scrollTo(scrollX, scrollY)
		} else if (target) {
			target.scrollIntoView();
		} else {
			window.scrollTo(0,0);
		}

		if (window.location.pathname.endsWith('/programme.html')) {
			openTab(0, null, true);
		}
	});

	/* Update last url browsed */
	OMA_LAST_URL = url;

	/* Prevent default navigation */
	if (e.currentTarget) {
		e.preventDefault();
		e.stopPropagation();
		return false
	}
}

/* On user history action */
window.addEventListener('popstate', function(event) {
	if (event.state && event.state.url) {
		internalNavigation(event.state.url, false, event.state.scrollX, event.state.scrollY);
	}
});

/* Store as global function. If the player successfuly loads, it will call this function */

window.advancedSite = false;
window.enableAdvancedSite = enableAdvancedSite;


/* Fix the menu */
var menu = document.getElementById('mainmenu');
document.getElementById("mainbody").style['margin-top'] = menu.offsetHeight + 'px';
menu.style['position'] = 'fixed';
menu.style['top'] = '0';

window.addEventListener('resize', function(event) {
	document.getElementById("mainbody").style['margin-top'] = menu.offsetHeight + 'px';
}, true);
