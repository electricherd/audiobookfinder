// SecondScreen.dart
import 'package:adbflib/adbflib.dart';
import 'package:flutter/widgets.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter_spinkit/flutter_spinkit.dart';
import 'package:flutter/material.dart';

class SearchTab extends StatefulWidget {
  SearchTab({Key key, this.title}) : super(key: key);
  final String title;
  @override
  _SearchTabState createState() => _SearchTabState();
}

class _SearchTabState extends State<SearchTab> {
  int _findings = 0;

  bool _searching_path = false;
  bool _searching_peers = false;

  String _path = '';
  String _peer_id = '';

  Adbflib adbflib;
  // AnimationController animController;

  @override
  void initState() {
    super.initState();
    adbflib = Adbflib();
    Adbflib.setup();

    // animController = AnimationController(
    //   duration: Duration(milliseconds: 1200),
    // );
  }

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
            const SizedBox(height: 50),
            RaisedButton(
              color: _searching_peers ? Colors.greenAccent : Colors.lime,
              child: Text(
                'Start Peer Search',
                style: TextStyle(
                  color: Colors.white,
                ),
              ),
              onPressed: () {
                if (!_searching_peers) {
                  _findNewPeer();
                }
              },
            ),
            Text(
              'First peer found:',
            ),
            const SizedBox(height: 5),
            Text(
              '$_peer_id',
              style: TextStyle(
                fontFamily: "monospace",
                color: Colors.white,
              ),
            )
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

  void _findNewPeer() async {
    _searching_peers = true;
    setState(() {});
    int peer_int = await adbflib.findNewPeer();
    // it's int not uint
    if (peer_int < 0) {
      peer_int = -peer_int;
    }
    _peer_id = peer_int.toRadixString(16);
    _searching_peers = false;
    setState(() {});
  }
}
