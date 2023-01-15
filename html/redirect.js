function isPersonalPage(path) {
    const personal_pages = [
        "/timeline.html",
        "/editor.html",
        "/user.html"
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
        } else if(this.status == 401) {
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
    let user_uuid = getCookieValue("user_uuid");
    let session_uuid = getCookieValue("session_uuid");

    let xhr = new XMLHttpRequest();
    xhr.open("POST", "/api/users/" + user_uuid + "/sessions/" + session_uuid);
    xhr.onreadystatechange = onRequestStateChange;
    xhr.send();
}

validateAuth();
window.setInterval(validateAuth, 5000);
