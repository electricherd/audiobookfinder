// peer_tab.dart
import 'dart:convert';
import 'package:adbflib/adbflib.dart';
import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';

// https://flutter.dev/docs/development/data-and-backend/json
class UIListElement {
  final String peerid;
  final int finished;
  final int searched;

  UIListElement(this.peerid, this.finished, this.searched);
  UIListElement.fromJson(Map<String, dynamic> json)
      : peerid = json['peerid'],
        finished = json['finished'],
        searched = json['searched'];
  Map<String, dynamic> toJson() =>
      {
        'peerid': peerid,
        'finished': finished,
        'searched': searched,
      };
}


class PeerTab extends StatefulWidget {
  Adbflib adbflib;
  PeerTab(this.adbflib, {Key key}) : super(key: key);
  @override
  _PeerTabState createState() => _PeerTabState(adbflib);
}

class _PeerTabState extends State<PeerTab> with AutomaticKeepAliveClientMixin<PeerTab> {
  Adbflib _adbflib;
  _PeerTabState(this._adbflib);

  String _ownIdString = '';
  List<UIListElement> _uiList = [];

  @override
  void initState() {
    super.initState();
    final int ownIntId = _adbflib.getOwnPeerId();
    _ownIdString = i64AsU64ToString(ownIntId);
    // async looping
    this._startLoopNetMessaging();
  }


  @override
  bool get wantKeepAlive => true;
  @override
  Widget build(BuildContext context) {
    return Container(
      child: Scaffold(
        body: Align(
          alignment: Alignment.topCenter,
          child: Column(
            mainAxisAlignment: MainAxisAlignment.start,
            children: [
              const SizedBox(height: 20),
              Text(
                "Own Peer ID: $_ownIdString",
                style: TextStyle(
                  fontFamily: "monospace",
                  color: Colors.white,
                ),
              ),
              const Divider(
                height: 50,
                thickness: 2,
                color: Colors.white,
              ),
              const Text(
                "peers on network: ",
                style: TextStyle(
                  color: Colors.white,
                ),
              ),
              const SizedBox(height: 20),
              Expanded(
                child: ListView.builder(
                  itemBuilder: _buildPeerItem,
                  itemCount: _uiList.length,
                )
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildPeerItem(BuildContext context, int index) {
    return Card(
      child: Row(
        children: <Widget>[
          // to define a height
          const SizedBox(height: 60),
          Expanded (
            flex: 5,
            child: Text(_uiList[index].peerid,
              textAlign: TextAlign.center,
              style: TextStyle(
                fontFamily: "monospace",
                color: Colors.white,
              ),
            )
          ),
          Expanded (
              flex: 3,
              child: Text( (_uiList[index].finished < 0) ?
                         "" :
                         "analyzed ${_uiList[index].finished}",
                textAlign: TextAlign.center,
                style: TextStyle(
                  color: Colors.white,
                ),
              )
          ),
          Expanded (
              flex: 2,
              child: Text( (_uiList[index].finished < 0) ?
              "" :
              "of ${_uiList[index].searched}",
                textAlign: TextAlign.center,
                style: TextStyle(
                  color: Colors.white,
                ),
              )
          ),
        ],
      ),
    );
  }

  // https://flutter.dev/docs/development/data-and-backend/json
  void _startLoopNetMessaging() async {
    while (true) {
      final String uiJson = await _adbflib.getNetUiMessages();
      _uiList = (jsonDecode(uiJson) as List).map((m) => UIListElement.fromJson(m)).toList();
       setState(() {});
    }
  }


  // please ... this is a complete hack about
  // dart's non capability for u64, and some strange behavior with
  // numbers!!!
  String i64AsU64ToString(int number) {
    String out = '';
    if (number > 0) {
      out = number.toRadixString(16);
    } else {
      final lastHexDigit = (number % 16);
      number = -((number >> 4)^0xfffffffffffffff) -1;
      out = (number.toRadixString(16)) + ((lastHexDigit).toRadixString(16)) ;
    }
    return out;
  }
}