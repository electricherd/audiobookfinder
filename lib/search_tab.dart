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
                RaisedButton(
                  color: _searchingPath ? Colors.greenAccent : Colors.lime,
                  child: Text(
                    'Search with adbf',
                    style: TextStyle(
                      color: Colors.white,
                    ),
                  ),
                  onPressed: () {
                    if (!_searchingPath) {
                      _getDirPath();
                    }
                  },
                ),
    //            SpinKitWave(
    //              color: Colors.blue,
    //              size: 30.0,
    //              controller: animController,
    //            ),
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
    _path = await FilePicker.platform.getDirectoryPath();
    _findings = 0;
    _searchingPath = true;
    setState(() {});
    _findings = await _adbflib.fileCountGood(_path);
    _adbflib.sendSearchResults(0, _findings);
    _searchingPath = false;
    setState(() {});
  }
}
