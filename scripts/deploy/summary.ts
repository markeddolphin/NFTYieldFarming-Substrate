import fs from 'fs';
const tablemark = require('tablemark')

interface Summary {
  contracts: Object[],
  deployed: Record<string, any>
}

function writeSummary(file: string, summary: Summary) {
  let table = tablemark(summary.contracts)
  fs.writeFile(file, table, function (err) {
    if (err) return console.log(err);
  })
}

export { Summary, writeSummary}