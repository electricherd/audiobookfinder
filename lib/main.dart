import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:adbflib/adbflib.dart';

import 'license_tab.dart';
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
  Adbflib _adbflib;

  SearchTab _searchTab;
  PeerTab _peerTab;
  LicenseTab _licenseTab;

  @override
  void initState() {
    super.initState();
    _adbflib = Adbflib();
    Adbflib.setup();

    _searchTab = SearchTab(_adbflib);
    _peerTab = PeerTab(_adbflib);
    _licenseTab = LicenseTab();
  }

  @override
  Widget build(BuildContext context) {
    return DefaultTabController(
      length: 3,
      child: Builder(builder: (BuildContext context) {
        final TabController tabController = DefaultTabController.of(context);
        tabController.addListener(() {
          if (!tabController.indexIsChanging) {
            // To get index of current tab use tabController.index
          }
        });
        return Scaffold(
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
                  ),
                  Tab(
                      child: Align(
                        alignment: Alignment.center,
                        child: Text("Licenses"),
                      )
                  )
                ],
              ),
              title: Text('adfbfflutter'),
            ),
            body: TabBarView(
              children: [
              _searchTab,
              _peerTab,
              _licenseTab,
              ],
            ),
        );
    }));
  }
}
