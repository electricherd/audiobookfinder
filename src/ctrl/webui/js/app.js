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
            ws.send('start'); //+"guid_id");
            //ws.send( 'start', {id: '<!--UUID-->'} );
        });
        ws.bind('close', function(){
            $('#statusMessage').text("not connected");
           alert("Connection is closed...");
        });

        ws.bind('init', function(data){
           showPath(data);
        });

        ws.bind('searchPath', function(data){
           spinPath(data);
        });

        window.onbeforeunload = function(event) {
             socket.close();
        };
    } else {
        // The browser doesn't support WebSocket
        alert("WebSocket NOT supported by your Browser!");
    }

    $("#btn2").click(function(){
      //$("#MyActionTable").append("<tr><td>Action</td><td>TimeLastname</td></tr>");
    });

    // on all buttons it searches for spinner/span ...
    // todo: clearify how to make it nicer!!
    $('button').on('click', e => {
      let spinner = $(e.currentTarget).find('span')
      spinner.removeClass('d-none')
      setTimeout(_ => spinner.addClass('d-none'), 2000)
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

function spinPath(data) {
    let path_nr = data[0].nr;
    let on_off = data[1];

    let spinner = $('#path_obj' + path_nr).find('span');

    if (on_off === false) {
        spinner.removeClass('d-none');
        setTimeout(_ => spinner.addClass('d-none'), 1000);
    }
    // todo: on is still missed due to timing
}

function showPath(data) {
    // encapsulate paths json
    let paths = data.paths;

    let path_len = paths.length;
    for (let i=0; i < path_len; i++) {
        let obj_id =  "path_obj" + paths[i].nr;
        // create obj
        let new_el_html = "<tr id='" + obj_id + "'><td>"
                         + paths[i].name + "</td><td>"
                         +"<span class='spinner-border spinner-border-sm' role='status' aria-hidden='true'></span>"
                         + "</tr></td>";
        new_el = $.parseHTML(new_el_html);
        // append it as an object
        $("#paths_searched").append(new_el);
    }
}

