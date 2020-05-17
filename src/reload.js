const source = new EventSource("/sse");
source.onmessage = message => {
    if (message.data === "reload") {
        fetch(location.href)
            .then(response => response.text())
            .then(text => {
                document.documentElement.innerHTML = text;
            });
    }
};