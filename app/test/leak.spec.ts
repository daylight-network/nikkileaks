import { Gateway } from 'oasis-std';

import { Leak } from '../service-clients/leak';

import moment from 'moment';

jest.setTimeout(5000);

const description: string = "my big secret";
const message: string = "i love kimchi";

function getTimestamp (date: string): BigInt {
  return BigInt(moment(date, "M/D/YYYY H:mm").unix())
}

describe('Basic functionality', () => {
  let pastMsgService: Leak;
  let futureMsgService: Leak;
  let timeInPast: BigInt = BigInt(
    moment().subtract(1, 'hour').unix()
  );
  let timeInFuture: BigInt = BigInt(
    moment().add(1, 'hour').unix()
  );

  // create a gateway to the oasis node
  let gw: Gateway = new Gateway(
    'http://localhost:1234',
    'AAAAGYHZxhwjJXjnGEIiyDCyZJq+Prknbneb9gYe9teCKrGa',
  );

  beforeAll(async () => {
    // post a message that's already released
    pastMsgService = await Leak.deploy(gw, {
      publicDescription: description,
      message: message,
      messageReleaseTime: timeInPast,
    });

    // post a message that will be released in the future
    futureMsgService = await Leak.deploy(gw, {
      publicDescription: description,
      message: message,
      messageReleaseTime: timeInFuture,
    });
  });

  it('deploys', async () => {
    expect(pastMsgService).toBeTruthy();
    expect(futureMsgService).toBeTruthy();
  });

  it('receive message released in the past', async () => {
      let pastMsg = await pastMsgService.message();
      expect(pastMsg).toBe(message);
  });


  it('does NOT receive message released in the future', async () => {
    await expect(futureMsgService.message())
      .rejects
      .toBe('Message is not yet released.')
  });

  it('access public description on released *and* unreleased leaks', async () => {
    let desc1 = await pastMsgService.getPublicDescription();
    expect(desc1).toBe(description);
    let desc2 = await futureMsgService.getPublicDescription();
    expect(desc2).toBe(description);
  });

  it('can update release time of not-yet-released messages', async () => {
    let _ = await futureMsgService.changeReleaseTime({newTime: timeInPast})
    let msg = await futureMsgService.message();
    expect(msg).toBe(message);
  });


  it('can NOT update release time of already-released messages', async () => {
    await expect(pastMsgService.changeReleaseTime({newTime: timeInFuture}))
      .rejects
      .toBe('Cannot update release time of already-released message.')
  });

  afterAll(async () => {
    await gw.disconnect();
  });
});
