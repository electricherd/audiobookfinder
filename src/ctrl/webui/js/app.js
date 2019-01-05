window.onload = APPStart;

// Page onload event handler
function APPStart() {
    state = false;

    if ("WebSocket" in window) {
        var ws = new WebSocket("ws://<!--WEBSOCKET-->/ws");

        ws.onopen = function() {
            alert ("Connected");
            $('#hello_message').text("Connected");
            ws.send("yes");
        };

        ws.onmessage = function (evt) {
            var received_msg = evt.data;
        };

        ws.onclose = function() {
             alert("Connection is closed...");
        };

        window.onbeforeunload = function(event) {
             socket.close();
        };
    } else {
        // The browser doesn't support WebSocket
        alert("WebSocket NOT supported by your Browser!");
    }

    // program checks if led_state button was clicked
    $('#state').click(function() {
        alert ("click");
        // changes local led state
        if (led_state == true){
            $('#on').hide();
            $('#off').show();
            state = false;
            ws.send("ON");
        }
        else{
            $('#off').hide();
            $('#on').show();
            state = true;
            ws.send("OFF");
        }
    });
}
