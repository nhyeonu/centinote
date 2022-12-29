function submitLogin() {
    let xhr = new XMLHttpRequest();
    xhr.open("POST", "/api/login");
    xhr.setRequestHeader("Accept", "application/json");
    xhr.setRequestHeader("Content-Type", "application/json");

    xhr.onreadystatechange = function() {
        if(this.readyState == 4 && this.status > 99 && this.status < 300) {
            window.location.replace("/timeline.html");
        }
    };

    const data = {};
    data.username = document.getElementById("username").value;
    data.password = document.getElementById("password").value;

    xhr.send(JSON.stringify(data));
}
