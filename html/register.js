function setFormWarning(description, focus_element) {
    const container = document.getElementById("warning-container");
    const paragraph = document.getElementById("warning-paragraph");

    container.hidden = false;
    paragraph.innerHTML = description;

    focus_element.focus();
}

function submitRegister() {
    const username_input_element = document.getElementById("username");
    const password_input_element = document.getElementById("password");
    const password_confirm_input_element = document.getElementById("password-confirm");
    const submit_input_element = document.getElementById("submit");

    let xhr = new XMLHttpRequest();
    xhr.open("POST", "/api/users");
    xhr.setRequestHeader("Accept", "application/json");
    xhr.setRequestHeader("Content-Type", "application/json");

    xhr.onreadystatechange = function() {
        if(xhr.readyState == 4) {
            if(xhr.status > 99 && xhr.status < 300) {
                window.location.href = "/login.html";
            } else {
                setFormWarning(
                    "Something has gone wrong! " +
                    "Please contact the server admin if the problem persists.",
                    submit_input_element);
            }
        }
    };

    const data = {};
    data.username = username_input_element.value;
    data.password = password_input_element.value;

    if(data.username.length == 0) {
        setFormWarning("Username is required!", username_input_element);
        return;
    }

    if(data.username.length < 2) {
        setFormWarning("Username cannot be one character!", username_input_element);
        return;
    }

    if(data.password.length == 0) {
        setFormWarning("Password is required!", password_input_element);
        return;
    }

    if(data.password.length < 6) {
        setFormWarning("Password must be at least 6 characters!", password_input_element);
        return;
    }

    if(password_input_element.value != password_confirm_input_element.value) {
        setFormWarning("Passwords do not match!", password_confirm_input_element);
        return;
    }

    xhr.send(JSON.stringify(data));
}
