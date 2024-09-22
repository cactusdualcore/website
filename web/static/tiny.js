const x = document.getElementById("net");
x.addEventListener('click', ev => {
    fetch("/awesome", {
        headers: {
            "CactusDualcore-Token": "AQAAAAVhbGluYQ==",
        }
    });
})