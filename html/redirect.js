function isPersonalPage(path) {
    const personal_pages = [
        "/timeline.html",
        "/editor.html"
    ];

    return personal_pages.includes(path);
}

function isAuthPage(path) {
    const auth_pages = [
        "/login.html",
        "/register.html"
    ];
    
    return auth_pages.includes(path);
}

function redirect(is_user_logged_in) {
    const is_auth_page = isAuthPage(window.location.pathname);
    const is_personal_page = isPersonalPage(window.location.pathname);

    if(is_user_logged_in && !is_personal_page) {
        window.location.replace("/timeline.html");
    }

    if(!is_user_logged_in && !is_auth_page) {
        window.location.replace("/login.html");
    }
}

function onRequestStateChange() {
    if(this.readyState == 4) {
        if(this.status > 99 && this.status < 300) {
            redirect(true);
        } else {
            redirect(false)
        }
    }
}

function getCookieValue(target_name) {
    const cookies = document.cookie.split("; ");
    const target_cookie = cookies.find((cookie) => cookie.startsWith(target_name + "="));
    const value = target_cookie?.split("=")[1];
    return value;
}

let xhr = new XMLHttpRequest();
xhr.open("GET", "/api/users/" + getCookieValue("user_uuid"));
xhr.onreadystatechange = onRequestStateChange;
xhr.send();
