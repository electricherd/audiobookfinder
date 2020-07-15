window.onload = APPStart;

var known_paths = [];

// Page onload event handler
function APPStart() {
    state = false;
    // guid_id = "<!--UUID-->";
    guid_id = "devel";
    $(document).ready(function(){
        // keep it for later with different id
        //$('#peer_page').load('peer_page.html');

        if ("WebSocket" in window) {
            // using something from other js - seems fine
            var ws = new FancyWebSocket("ws://<!--WEBSOCKET_ADDR-->:<!--PORT_WEBSOCKET-->/ws");

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
               updateNetView(data);
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
        let new_el_html = "<tr id='" + obj_id + "'><td class='col-xs-3'>"
                         + paths[i].name + "</td><td>"
                         +"<span class='d-none spinner-border spinner-border-sm' role='status' aria-hidden='true'></span>"
                         + "</td></tr>";
        new_el = $.parseHTML(new_el_html);
        // append it as an object but wait since div creation need little time
        $("#paths_searched").append(new_el);
    }
}

function updateNetView(data) {
    if (data.view === 'add') {
        let peer_id = data.cnt.id;
        let addresses = data.cnt.addr;

        let obj_id =  "host_obj_" + peer_id;
        let tooltip = "";
        for(let i = 0; i < addresses.length; i++){
            tooltip += "=" + addresses[i] + "=";
        }
        // create obj
        let new_el_html = "<tr id='" + obj_id + "' "
                        // todo: fix html tooltip
                         + "data-toggle='tooltip' data-placement='bottom' data-html='true' "
                         + "title='Adresses: "
                         + tooltip
                         + "'><td class='col-xs-3'>"
                         + peer_id
                         + "</td></tr>";
        new_el = $.parseHTML(new_el_html);
        // append it as an object
        $("#found_peers").append(new_el);
        $('#' + obj_id).hide().fadeIn(500);
    }
    else if (data.view === 'remove') {
        // delete object
        let peer_id = data.cnt;

        let obj_id =  "host_obj_" + peer_id;
        $('#' + obj_id).fadeOut(1500).remove();
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
      document.getElementById("overlay").style.display = "block";
}