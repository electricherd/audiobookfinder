window.onload = APPStart;

// Page onload event handler
function APPStart() {
    state = false;
    // guid_id = "<!--UUID-->";
    guid_id = "devel";

    $(document).ready(function(){
    if ("WebSocket" in window) {
        // var ws = new WebSocket("ws://<!--WEBSOCKET-->/ws");
        var ws = new WebSocket("ws://localhost:8088/ws");

        ws.onopen = function() {
            $('#statusMessage').text("connected");
            ws.send("/start "+"guid_id");
        };

        ws.onmessage = function (evt) {
            var received_msg = evt.data;
            $("#myItemTable").prepend("<tr><td>" + evt.data
                                + "</td><td>TimeLastname</td></tr>");
        };

        ws.onclose = function() {
            $('#statusMessage').text("not connected");
           alert("Connection is closed...");
        };

        window.onbeforeunload = function(event) {
             socket.close();
        };
    } else {
        // The browser doesn't support WebSocket
        alert("WebSocket NOT supported by your Browser!");
    }

    $("#btn2").click(function(){
      $("#MyActionTable").append("<tr><td>Action</td><td>TimeLastname</td></tr>");
    });

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
  });
}
