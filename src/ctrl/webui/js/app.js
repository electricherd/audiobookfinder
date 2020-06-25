window.onload = APPStart;

// Page onload event handler
function APPStart() {
    state = false;
    // guid_id = "<!--UUID-->";
    guid_id = "devel";

    $(document).ready(function(){
    if ("WebSocket" in window) {
        // using something from other js - seems fine
        var ws = new FancyWebSocket("ws://localhost:8088/ws");

        // the usual suspects
        ws.bind('open', function(){
            $('#statusMessage').text("connected");
            ws.send("/start "+"guid_id");
        });
        ws.bind('close', function(){
            $('#statusMessage').text("not connected");
           alert("Connection is closed...");
        });

        ws.bind('animate', function(all){
           $("#paths_searched").append("<tr><td><span class='spinner-border spinner-border-sm' role='status' aria-hidden='true'></span></td></tr>");
        });

        ws.bind('refresh', function(nr){
            $("#paths_searched").append("<tr><td><span class='spinner-border spinner-border-sm' role='status' aria-hidden='true'></span></td></tr>");
        })

        ws.bind('refresh', function(nr){
            $("#paths_searched").append("<tr><td><span class='spinner-border spinner-border-sm' role='status' aria-hidden='true'></span></td></tr>");
        })


        window.onbeforeunload = function(event) {
             socket.close();
        };
    } else {
        // The browser doesn't support WebSocket
        alert("WebSocket NOT supported by your Browser!");
    }

    $("#btn2").click(function(){
      //$("#MyActionTable").append("<tr><td>Action</td><td>TimeLastname</td></tr>");
      $("#paths_searched").append("<tr><td><span class='spinner-border spinner-border-sm' role='status' aria-hidden='true'></span></td></tr>");
    });

    // on all buttons it searches for spinner/span ...
    // todo: clearify how to make it nicer!!
    $(() => {
      $('button').on('click', e => {
        let spinner = $(e.currentTarget).find('span')
        spinner.removeClass('d-none')
        setTimeout(_ => spinner.addClass('d-none'), 2000)
      })
    })

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
