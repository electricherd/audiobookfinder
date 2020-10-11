import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:adbflib/adbflib.dart';

import 'search_tab.dart';
import 'peer_tab.dart';

void main() => runApp(MyApp());

class MyApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'adbfflutter',
      theme: ThemeData(
        primarySwatch: Colors.blue,
        brightness: Brightness.dark,
      ),
      home: MyHomePage(),
    );
  }
}

class MyHomePage extends StatefulWidget {
  MyHomePage({Key key}) : super(key: key);
  @override
  _MyHomePageState createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  Adbflib adbflib;

  @override
  void initState() {
    super.initState();
    adbflib = Adbflib();
    Adbflib.setup();
  }

  @override
  Widget build(BuildContext context) {
    return DefaultTabController(
        length: 2,
        child: Scaffold(
          appBar: AppBar(
            bottom: TabBar(
              tabs: [
                Tab(
                    child: Align(
                      alignment: Alignment.center,
                      child: Text("Search"),
                    )
                ),
                Tab(
                    child: Align(
                      alignment: Alignment.center,
                      child: Text("Network"),
                    )
                )
              ],
            ),
            title: Text('adfbfflutter'),
          ),
          body: TabBarView(
            children: [
              SearchTab(adbflib),
              PeerTab(adbflib),
            ],
          ),
        ),
      );
  }
}
