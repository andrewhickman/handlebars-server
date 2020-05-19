const source = new EventSource("/sse");
source.onmessage = message => {
    if (message.data === "reload_value") {
        fetch(location.href)
            .then(response => response.text())
            .then(text => {
                document.documentElement.innerHTML = text;
            });
    } else if (message.data === "reload_page") {
        location.reload();
    }
};