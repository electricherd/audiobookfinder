window.onload = APPStart;

var known_paths = [];
var path_ui_nr = 0;
var modal_dirs = [];

// Page onload event handler
function APPStart() {
    state = false;
    guid_id = "<!---PEER_HASH--->";
    max_paths = <!---PATHS_MAX--->;
    version_string = "<!---VERSION--->";

    $(document).ready(function(){
        // keep it for later with different id
        //$('#peer_page').load('peer_page.html');

        if ("WebSocket" in window) {
            // using something from other js - seems fine
            var ws = new FancyWebSocket("ws://<!---WEBSOCKET_ADDR--->:<!---PORT_WEBSOCKET--->/ws");

            // the usual suspects
            ws.bind('open', function(){
                $('#statusMessage').text("connected");
                ws.send('ready');
                // register hash UUID
            });
            ws.bind('close', function(){
                gracefullyClose();
            });

            ws.bind('init_paths', function(data){
               for(let i = 0; i < data.length; i++){
                  addModalPathSelector(data[i]);
               }
            });

            ws.bind('start', function(data){
               showPath(data);
            });

            ws.bind('searching', function(data){
               spinPath(data);
            });

            ws.bind('update', function(data){
               updateNetView(data);
            });

            ws.bind('rest_dirs', function(data){
               onRESTDir(data);
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

            // modal button events
            $('#modal_add').click(function() {
                handle_path_add_button();
            });
            $('#modal_sub').click(function(){
                handle_path_sub_button();
            });
            $('#modal_close').click(function(){
                ws.send('start', modal_dirs);
            });
            // dynamic content problem
            $('#modal_path_table').on('click', 'div > button.dirDropper',  function(event){
              //event.preventDefault();
              event.stopPropagation();
              // read nr from suffix
              let splitter = this.id.split("_")[1];
              let nr = parseInt(splitter);
              ws.send('rest_dir', {'nr': nr, 'dir': modal_dirs[nr]});
              this.dropdown('dispose');
            });

            // STARTUP with modal
            $('#modal_dir_chooser').modal('show');

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
                         + helper_win_canonical(paths[i].name) + "</td><td>"
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
        let new_el_html =  '<tr id="' + obj_id + '" '
                         + 'data-toggle="tooltip" data-placement="bottom" data-html="true" '
                         + 'title="Adresses: '
                         + tooltip
                         + '"><td class="col-xs-3">'
                         + '<button id="netbutton_' + peer_id + '" class="btn btn-light text-monospace" '
                         + 'value="' + guid_id + '"'
                         + 'onClick="netButtonClick(this.id, this.value)">'
                         + peer_id
                         + '</button>'
                         + '</td><td id="result_' + peer_id + '">'
                         + '<div class="spinner-border spinner-border-sm text-success" role="status">'
                         + ' <span class="sr-only"></span>'
                         + '</div>'
                         + '</td></tr>';
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
    else if (data.view === 'finished') {
        let peer_id = data.cnt.peer;
        if (peer_id === guid_id) {
            // myself
            let new_el = html_build_finished_peer(data.cnt.data);
            // set text in peer_page
            $('#own_finished').html(new_el);
            // set tooltip
            $('#own_header').attr('title',new_el);
        } else {
            // foreign peer results
            let new_el = html_build_finished_peer(data.cnt.data);
            let first_result = data.cnt.data.nr_found_songs;
            let new_text =  '<div data-toggle="tooltip" data-placement="bottom" data-html="true"'
                          + '     title="' + new_el + '">[' + first_result + '] audio files found </div>';
            $('#result_' + peer_id + ' div').replaceWith(new_text);
        }
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

function addModalPathSelector(path_name) {
    let name = "home";
    if (path_name === undefined) {
        path_name = "";
    } else {
        name = helper_extractLastDir(path_name);
    }
    modal_dirs[path_ui_nr] = path_name;
    let path_string = ('0' + path_ui_nr).slice(-2);
    let new_selector =  '  <div id="dropdowndiv_' + path_string + '" class="dropdown" style="padding: 5px;">'
                      + '   <button class="btn btn-secondary dropdown-toggle dirDropper" id="dropmenu_' + path_string + '"'
                      + '           data-toggle="dropdown" aria-haspopup="true" aria-expanded="false"'
                      + '           style="max-height:50vh; overflow-y:auto;"'
                      + '   ><i>' + name + '</i><span class="sr-only">(current)</span></button>'
                      + '   <div id="dropdown_' + path_string + '"'
                      + '        class="dropdown-menu" aria-labelled-by="dropdownMenuButton"'
                      + '        style="transform: translate3d(0px,0px,0px);">'
                      + '   </div>'
                      + '  </div>';
    $("#modal_path_table").append(new_selector);
    // increase counter for new selector
    path_ui_nr += 1;
}

function onRESTDir(data) {
  // stop path spinners
  let nr = data.nr;
  let dirs_len = data.dirs.length;

  let path_string = ('0' + nr).slice(-2);
  let dropdown_menu = $("#dropdown_" + path_string);
  // trick: avoid that very first item often is above content
  //        in a bit longer lists by adding a divider ;-)
  dropdown_menu.append(' <div class="dropdown-divider"></div>')
  for (let i=0; i < dirs_len; i++) {
      let name = "";
      if (i === 0) {
         // first entry can be the ".." directory
         if (dirs_len > 1) {
            // if 1st is completly contained in the 2nd, then it's the ".."
            if (data.dirs[1].includes(data.dirs[0])) {
                // hide parent as ".."
                name = "..";
            } else {
                name = helper_extractLastDir(data.dirs[0]);
            }
         } else {
            // it's not always the ".."
            if (modal_dirs[nr].includes(data.dirs[0])) {
                name = "..";
            } else {
                name = helper_extractLastDir(data.dirs[0]);
            }
         }
      } else {
         name = helper_extractLastDir(data.dirs[i]);
      }
      let new_link =  '     <a class="dropdown-item" href="#"'
                    + '       onClick="uiUpdateDropMenu(' + nr + ',\'' + encodeURIComponent(data.dirs[i]) + '\');"'
                    + '      >' + name + '</a>';
      dropdown_menu.append(new_link);
      dropdown_menu.dropdown('update');
      dropdown_menu.dropdown('toggle');
  }
}

function uiUpdateDropMenu(nr, new_dir) {
    let path_string = ('0' + nr).slice(-2);
    let decode_dir = decodeURIComponent(new_dir);
    // update title
    $("#dropmenu_" + path_string).html(helper_extractLastDir(decode_dir) + '<span class="caret"></span>');
    modal_dirs[nr] = decode_dir;
    // remove old entries
    $("#dropdown_" + path_string).children().remove();
}

function handle_path_add_button() {
    if (path_ui_nr < max_paths) {
        addModalPathSelector(undefined);
        // grey out if end is reached
        if (path_ui_nr === max_paths) {
            $('#modal_add').prop('disabled',true).css('opacity',0.5);
        }
        // validate sub button
        $('#modal_sub').prop('disabled',false).css('opacity',1.0);
    }
}

function handle_path_sub_button() {
    if (path_ui_nr > 0) {
        path_ui_nr -= 1;
        modal_dirs.pop();
        let path_string = ('0' + path_ui_nr).slice(-2);
        $("#dropdowndiv_" + path_string).remove();
        if (path_ui_nr === 1) {
            $('#modal_sub').prop('disabled',true).css('opacity',0.5);
        }
        $('#modal_add').prop('disabled',false).css('opacity',1.0);
    }
}

function html_build_finished_peer(data) {
    let searched_files = data.nr_searched_files;
    let found_audio = data.nr_found_songs;
    let duplicates = data.nr_internal_duplicates;
    let size_data = data.size_of_data_in_kb;
    let html =   '<table>'
               + ' <tr><td>files searched:</td><td>' + searched_files + '</td><td> audio files analyzed:</td><td>' + found_audio + ' </td></tr>'
               + ' <tr><td>duplicates:</td><td>' + duplicates + '</td><td colspan=2>with ' + size_data + 'kb of data to send over network</td></tr>'
               + '</table>';
    return html;
}

function helper_extractLastDir(dir_path) {
    // should be platform independent, but's I can't check all,
    // especially windows first "canonical" form is difficult
    return dir_path.split(/.*[\/|\\]/)[1];
}

function helper_win_canonical(dir_path) {
    // clear of \\?\ prefix in windows
    // https://stackoverflow.com/questions/50322817/how-do-i-remove-the-prefix-from-a-canonical-windows-path
    // from rust but to be displayed nicely
    return dir_path.replace(/\\\\\?\\/g, '');
}
