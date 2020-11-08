// license_tab.dart
import 'package:flutter/widgets.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart' show rootBundle;

class LicenseTab extends StatefulWidget {
  LicenseTab({Key key}) : super(key: key);
  @override
  _LicenseTabState createState() => _LicenseTabState();
}

class _LicenseTabState extends State<LicenseTab> with AutomaticKeepAliveClientMixin<LicenseTab> {
  List _displayBlock = new List<String>();

  @override
  void initState() {
    final List _licensesSrc = ['LICENSE',
                              'licenses/flutter_file_picker.txt',
                              'licenses/js_bootstrap.txt',
                              'licenses/js_ws_events_dispatcher.txt',
                              'licenses/rs_bktree-rs.txt'];
    for (var i=0; i < _licensesSrc.length; i++) {
      rootBundle.loadString(_licensesSrc[i]).then((String text){
        _displayBlock.add(text);
        if (i < _licensesSrc.length-1) {
          _displayBlock.add("----------------------------------------------");
        } else {
          // only update on last entry
          setState(() {});
        }
      });
    }
  }
  @override
  bool get wantKeepAlive => true;
  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: <Widget> [
          ListView.builder(
            physics: NeverScrollableScrollPhysics(),
            shrinkWrap: true,
            itemCount:_displayBlock.length,
            itemBuilder: (context,index){
              return  Text(
                _displayBlock[index],
                style: TextStyle(
                  fontFamily: "monospace",
                  fontSize: 10.0,
                )
              );
            }),
        ]
      ),
    );
  }
}
