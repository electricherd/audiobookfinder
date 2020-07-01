window.onload = APPStart;

var known_paths = [];

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
            ws.send('start');
            // register hash UUID
        });
        ws.bind('close', function(){
            gracefullyClose();
        });

        ws.bind('init', function(data){
           showPath(data);
        });

        ws.bind('searching', function(data){
           spinPath(data);
        });

        ws.bind('update', function(data){
           updateView(data);
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
    let on_off = data[1];

    if (data[0].type === 'net') {
       let spinner = $('#host_search_progress').find('span');

       if (on_off === true) {
           spinner.removeClass('d-none');
       } else {
           //spinner.removeClass('d-none');
           setTimeout(_ => spinner.addClass('d-none'), 100);
       }
    } else {
        let path_nr = data[0].cnt.nr
        let spinner = $('#path_obj' + path_nr).find('span');

        if (on_off === true) {
            spinner.removeClass('d-none');
        } else {
            //setTimeout(_ => spinner.addClass('d-none'), 100);
            spinner.addClass('d-none');
        }
    }
}

function showPath(data) {
    // encapsulate paths json
    let paths = data.paths;

    let path_len = paths.length;
    for (let i=0; i < path_len; i++) {
        let path_nr = paths[i].nr;
        // register for graceful closing
        known_paths.push(path_nr);
        let obj_id =  "path_obj" + path_nr;
        // create obj
        let new_el_html = "<tr id='" + obj_id + "'><td>"
                         + paths[i].name + "</td><td>"
                         +"<span class='d-none spinner-border spinner-border-sm' role='status' aria-hidden='true'></span>"
                         + "</td></tr>";
        new_el = $.parseHTML(new_el_html);
        // append it as an object
        $("#paths_searched").append(new_el);
    }
}

function updateView(data) {
    if (data.view === 'host') {
        let peer_id = data.cnt;
        let obj_id =  "host_obj_" + peer_id;
        // create obj
        let new_el_html = "<tr id='" + obj_id + "'><td>"
                         + peer_id
                         + "</td></tr>";
        new_el = $.parseHTML(new_el_html);
        // append it as an object
        $("#MyIPTable").append(new_el);
    } else {
        console.log("The view '" + data.view + "' is not implemented yet!");
    }
}

function gracefullyClose() {
      // stop host spinner
      let spinnerSearchHost = $('#host_search_progress').find('span');
      spinnerSearchHost.addClass('d-none');
      // stop path spinners
      for (let i=0; i < known_paths.length; i++) {
          let spinnerPath = $('#path_obj' + known_paths[i]).find('span');
          spinnerPath.addClass('d-none');
      }

      $('#statusMessage').text("not connected");
      alert("Connection is closed...");
}