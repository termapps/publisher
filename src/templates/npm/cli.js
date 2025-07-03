#!/usr/bin/env node

const { maybeInstall, run } = require("./binary");

maybeInstall().then(() => run());
