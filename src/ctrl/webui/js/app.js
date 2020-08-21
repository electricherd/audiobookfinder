window.onload = APPStart;

var known_paths = [];
var path_ui_nr = 0;
var modal_dirs = [];

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

            ws.bind('rest_dirs', function(data){
               alert("rest_dirs");
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

            // add first entry for modal path dialog
            addModalPathSelector();


            $(".dirButton" ).click( function() {
                let splitter = this.name.split("_")[1];
                let nr = parseInt(splitter);
                ws.send('rest_dir', modal_dirs[nr]);
            } );

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
                         + "<button id='netbutton_" + peer_id + "' class='btn btn-light text-monospace' "
                         + "value='" + guid_id + "'"
                         + "onClick='netButtonClick(this.id, this.value)'>"
                         + peer_id
                         + "</button>"
                         + "</td><td id='count_" + peer_id + "'>"
                         + "<div class='spinner-border spinner-border-sm text-success' role='status'>"
                         + " <span class='sr-only'></span>"
                         + "</div>"
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
    }
    else if (data.view === 'count') {
        // delete object
        let peer_id = data.cnt.peer;
        let count = data.cnt.count;

        let new_text = "<div>[" + count + "]</div>";
        $('#count_' + peer_id + ' div').replaceWith(new_text);
    }
    else {
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

function netButtonClick(button_peer, guid) {
    // get frame (table id)
    let peer = button_peer.split("_")[1];

    // if exists go back
    if ($("#frame_" + peer).length)
        return


    // collapse old entry
    $("#frame_"+peer).collapse('toggle');

    // create entry
    let new_foreign_row = '<div id="frame_' + peer + '"><div class="link rounded">Foreign: ' + peer
                        + '</div><div class="submenu" id="page_' + peer +  '"></div></div>';
    $("#accordion").append(new_foreign_row);
}

function addModalPathSelector() {
    // increase counter for new selector
    modal_dirs[path_ui_nr] = "";
    let path_string = ('0' + path_ui_nr).slice(-2);
    let new_selector =  '<tr><td>'
                      + '  <input type="text" class="form-control" id="dir_'+ path_string +'"'
                      + '   placeholder=" ... ">'
                      + '</td>'
                      + '<td>'
                      + '  <input type="button" class="btn btn-light text-monospace dirButton"'
                      + '          value="Select Path' + path_string + '"'
                      + '          name="btn_' + path_ui_nr + '">'
                      + '</td></tr>';
    $("#modal_path_table").append(new_selector);
    path_ui_nr += 1;
}
