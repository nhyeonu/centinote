function getCookieValue(target_name) {
    const cookies = document.cookie.split("; ");
    const target_cookie = cookies.find((cookie) => cookie.startsWith(target_name + "="));
    const value = target_cookie?.split("=")[1];
    return value;
}

function logout() {
    let user_uuid = getCookieValue("user_uuid");
    let session_uuid = getCookieValue("session_uuid");

    let xhr = new XMLHttpRequest();
    xhr.open("DELETE", "/api/users/" + user_uuid + "/sessions/" + session_uuid);

    xhr.onreadystatechange = function() {
        if(this.readyState == 4 && this.status > 99 && this.status < 300) {
            window.location.href = "/login.html";
        }
    };

    xhr.send();
}
