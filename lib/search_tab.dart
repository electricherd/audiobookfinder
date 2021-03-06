// SecondScreen.dart
import 'package:adbflib/adbflib.dart';
import 'package:flutter/widgets.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter_spinkit/flutter_spinkit.dart';
import 'package:flutter/material.dart';

class SearchTab extends StatefulWidget {
  Adbflib adbflib;
  SearchTab(this.adbflib, {Key key}) : super(key: key);
  @override
  _SearchTabState createState() => _SearchTabState(adbflib);
}

class _SearchTabState extends State<SearchTab> with AutomaticKeepAliveClientMixin<SearchTab> {
  Adbflib _adbflib;
  _SearchTabState(this._adbflib);

  int _findings = 0;
  bool _searchingPath = false;
  String _path = '';

  @override
  bool get wantKeepAlive => true;
  @override
  Widget build(BuildContext context) {
    return Container(
        child: Scaffold(
          body: Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Padding(
                    padding: EdgeInsets.all(30),
                child: Text(
                  'You can search local audio(book) files here by selecting '
                  'a single search path using the search button below.\n'
                  'The search '
                  'results will be provided to other adbf-clients on the same '
                  'local network over time, and it will listen to results '
                  'from other clients as well and show them in the '
                  'network tab.\n'
                  'See the license tab for all licenses used.',
                  textAlign: TextAlign.start)
                ),
                Stack(
                  alignment: Alignment.center,
                  children: <Widget> [
                    Opacity(
                      opacity: _searchingPath ? 0.0 : 1.0,
                      child:
                        RaisedButton(
                          color: Colors.lime,
                          child: Text(
                            'Search with adbf',
                            style: TextStyle(
                              color: Colors.black,
                            ),
                          ),
                          onPressed: () {
                            if (!_searchingPath) {
                              _getDirPath();
                            }
                          },
                        ),
                    ),
                   Opacity(
                     opacity: _searchingPath ? 1.0 : 0.0,
                     child: SpinKitWave(
                      color: Colors.blue,
                      size: 30.0,
                     ),
                   ),
                ]),
                const SizedBox(height: 30),
                Text(
                  'A number of $_findings audio files have been analyzed!',
                ),
            ],
        ),
      ),
    ),
    );
  }

  void _getDirPath() async {
    final String oldPath = _path;
    _path = await FilePicker.platform.getDirectoryPath();
    if (_path.isNotEmpty) {
      _findings = 0;
      _searchingPath = true;
      setState(() {});
      _findings = await _adbflib.fileCountGood(_path);
      _searchingPath = false;
    }
    setState(() {});
  }
}
