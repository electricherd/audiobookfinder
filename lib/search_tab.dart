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
  Adbflib adbflib;
  _SearchTabState(this.adbflib);

  int _findings = 0;
  bool _searching_path = false;
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
                  color: _searching_path ? Colors.greenAccent : Colors.lime,
                  child: Text(
                    'Search with adbf',
                    style: TextStyle(
                      color: Colors.white,
                    ),
                  ),
                  onPressed: () {
                    if (!_searching_path) {
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
                  'A number of $_findings audio files have been found!',
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
    _searching_path = true;
    setState(() {});
    _findings = await adbflib.fileCountGood(_path);
    _searching_path = false;
    setState(() {});
  }
}
