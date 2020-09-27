import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:adbflib/adbflib.dart';
import 'package:flutter/widgets.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter_spinkit/flutter_spinkit.dart';

void main() => runApp(MyApp());

class MyApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Adbflib Flutter',
      theme: ThemeData(
        primarySwatch: Colors.blue,
        brightness: Brightness.dark,
      ),
      home: MyHomePage(title: 'Adbflib Flutter Demo'),
    );
  }
}

class MyHomePage extends StatefulWidget {
  MyHomePage({Key key, this.title}) : super(key: key);
  final String title;
  @override
  _MyHomePageState createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  int _findings = 0;
  bool _searching = false;
  String _path = '';
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
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.title),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            RaisedButton(
              color: _searching ? Colors.greenAccent : Colors.yellow,
              child: Text(
                'Search with adbf',
                style: TextStyle(
                  color: Colors.white,
                ),
              ),
              onPressed: () {
                if (!_searching) {
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
            )
          ],
        ),
      ),
    );
  }

  void _getDirPath() async {
    _path = await FilePicker.platform.getDirectoryPath();
    _findings = 0;
    _searching = true;
    setState(() {});
    _findings = await adbflib.fileCountGood(_path);
    _searching = false;
    setState(() {});
  }
}
