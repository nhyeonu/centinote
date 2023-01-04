function logout() {
    let xhr = new XMLHttpRequest();
    xhr.open("DELETE", "/api/session");

    xhr.onreadystatechange = function() {
        if(this.readyState == 4 && this.status > 99 && this.status < 300) {
            window.location.href = "/login.html";
        }
    };

    xhr.send();
}
