import { Gateway } from 'oasis-std';
import { Address } from 'oasis-std';

import { Leak } from '../service-clients/leak';

import moment from 'moment';

function makeGateway () {
  return new Gateway(
    'https://gateway.devnet.oasiscloud.io',
    'AAAAGYHZxhwjJXjnGEIiyDCyZJq+Prknbneb9gYe9teCKrGa',
  );
}

async function main() {
  const gw: Gateway = makeGateway()

  const description: string = "my big secret";
  const message: string = "i love kimchi";

  const service = await Leak.deploy(gw, {
    publicDescription: description,
    message: message,
    messageReleaseTime: BigInt(moment().add(2, 'minutes').unix()),
  });

  console.log(`Deployed Leak at ${service.address.hex}`);

}

// Call this method, passing in your contract's address,
// if you'd like to test your message out.
async function testMessage (addr: Address) {
  const gw: Gateway = makeGateway()

  let rel = await Leak.connect(addr, gw);
  let msg = await rel.message();

  console.log('received!', msg);
}

main().catch(console.error);
