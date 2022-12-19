function submitPost(form) {
    let xhr = new XMLHttpRequest();
    xhr.open("POST", form.getAttribute("action"));

    xhr.setRequestHeader("Accept", "application/json");
    xhr.setRequestHeader("Content-Type", "application/json");

    var data = {};

    let inputs = form.getElementsByTagName("input");
    for (const input of inputs) {
        data[input.getAttribute("name")] = input.value;
    }
    
    xhr.send(JSON.stringify(data));
}
