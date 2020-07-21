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


            // dealing with accordion
            var Accordion = function(el, multiple) {
                this.el = el || {};
                this.multiple = multiple || false;

                var links = this.el.find('.link');

                links.on('click', {el: this.el, multiple: this.multiple}, this.dropdown)
            }
            Accordion.prototype.dropdown = function(e) {
                var $el = e.data.el;
                $this = $(this),
                $next = $this.next();

                $next.slideToggle();
                $this.parent().toggleClass('open');

                if (!e.data.multiple) {
                    $el.find('.submenu').not($next).slideUp().parent().removeClass('open');
                };
            }
            var accordion = new Accordion($('#accordion'), false);

            // click accordion.div.div to show first entry!!
            $("#accordion div div").click();

        } else {
            // The browser doesn't support WebSocket
            alert("WebSocket NOT supported by your Browser!");
        }
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
    if ($("#frame_" + peer).length)
        return

    // get frame (table id)
    let identificator_frame = guid.split("_")[1];

    // collapse old entry
    $("#frame_"+identificator_frame).collapse('toggle');

    // create entry
    let new_foreign_row = '<div id="frame_' + peer + '"><div class="link rounded">Foreign: ' + peer
                        + '</div><div class="submenu" id="page_' + peer +  '"></div></div>';
    $("#accordion").append(new_foreign_row);

    // load foreign page
    $("#page_" + peer).load("http://<!---WEBSOCKET_ADDR--->:8088/peer_page.html", function( response, status, xhr ) {
      if ( status == "error" ) {
        var msg = "Sorry but there was an error: ";
        $( "#page_" + peer).html( msg + xhr.status + " " + xhr.statusText );
      }
    });
}

