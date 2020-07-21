window.onload = APPStart;

var known_paths = [];

// Page onload event handler
function APPStart() {
    state = false;
    guid_id = "<!---PEER_HASH--->";

    $(document).ready(function(){
        // keep it for later with different id
        //$('#peer_page').load('peer_page.html');

        if ("WebSocket" in window) {
            // using something from other js - seems fine
            var ws = new FancyWebSocket("ws://<!---WEBSOCKET_ADDR--->:<!---PORT_WEBSOCKET--->/ws");

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

        // dynamic added content
        // https://www.tutorialrepublic.com/faq/how-to-bind-click-event-to-dynamically-added-elements-in-jquery.php
        // for buttons
//        $("#found_peers").on("click", ".nettd", function(event) {
//            console.log("that should have been the way: ",JSON.stringify(event,null, 4));
//        });

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
            spinner.addClass('d-none');
            spinner.replaceWith( "<svg width='1em' height='1em' viewBox='0 0 16 18'"
                               + " class='bi bi-circle-fill' fill='currentColor'"
                               + " xmlns='http://www.w3.org/2000/svg'>"
                               + " <circle cx='8' cy='8' r='8'/></svg>");
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
        let new_el_html = "<tr id='" + obj_id + "'><td class='col-xs-3 text-monospace' style='width: 90%'>"
                         + paths[i].name + "</td><td>"
                         +"<span class='d-none spinner-grow spinner-grow-sm col-xs-3 text-right' role='status' aria-hidden='true'></span>"
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
        // tooltip is the multi-address for that peer
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
                         + "'><td class='nettd col-xs-3'>"
                         + "<button id='netbutton_" + guid_id + "' class='btn btn-light' "
                         + "value='" + peer_id + "'"
                         + "onClick='netButtonClick(this.id, this.value)'>"
                         + peer_id
                         + "</button>"
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
      document.getElementById("overlay").style.display = "block";
}

function netButtonClick(guid, peer) {
    // if exists go back
    if ($("#peer_" + peer).length)
        return

    // get frame (table id)
    let identificator_frame = guid.split("_")[1];

    // collapse old entry
    $("#peer_"+identificator_frame).collapse('toggle');



    let new_foreign_row = '<tr><td class="align-top"><table class="table table-fixed">'
                        + '<thead><tr><td class="w-100 text-right text-white" style="background-color: #74A5ED">'
                        + '    <button'
                        + '    class="text-white btn btn-primary"'
                        + '    type="button"'
                        + '    style="background-color: #74A5ED"'
                        + '    data-toggle="collapse"'
                        + '    data-target="#peer_' + peer + '">Foreign: ' + peer + '</button>'
                        + '</td></tr></thead>'
                        + '<tbody style="height: 40vh;">'
                        + '  <tr><td style="border-top-width: 0px;padding: 0px 0px 0px 0px;">'
                        + '     <div id="peer_' + peer + '"'
                        + '          class="collapse show"'
                        + '           style="border: 2px solid #74A5ED; padding: 0px 0.75em 0px 0.75em;">'
                        + '     </div>'
                        + ' </td></tr>'
                        + '</tbody>'
                        + '</table></td></tr>';
    $("#peers_table_tbody").append(new_foreign_row);

    alert("guid: " + identificator_frame + " Peer: " + peer);
}

