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

function onRequestStateChange() {
    if(this.readyState == 4) {
        if(this.status > 99 && this.status < 300) {
            if(!isPersonalPage(window.location.pathname)) {
                window.location.href = "/timeline.html";
            }
        } else {
            if(!isAuthPage(window.location.pathname)) {
                window.location.href = "/login.html";
            }
        }
    }
}

function getCookieValue(target_name) {
    const cookies = document.cookie.split("; ");
    const target_cookie = cookies.find((cookie) => cookie.startsWith(target_name + "="));
    const value = target_cookie?.split("=")[1];
    return value;
}

function validateAuth() {
    let xhr = new XMLHttpRequest();
    xhr.open("POST", "/api/session");
    xhr.onreadystatechange = onRequestStateChange;
    xhr.send();
}

validateAuth();
window.setInterval(validateAuth, 5000);
