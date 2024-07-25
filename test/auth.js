const { init } = require('../index.js')

const client = init(480)

;(async () => {
    {
        const ticket = await client.auth.getSessionTicketWithSteamId(BigInt(123456))
        console.log('getSessionTicketWithSteamId: ', ticket)
        console.log('getSessionTicketWithSteamId: ', ticket.getBytes())

        ticket.cancel()
    }

    {
        const ticket = await client.auth.getSessionTicketWithIp('192.168.0.5:1234')
        console.log('getSessionTicketWithSteamId: ', ticket)
        console.log('getSessionTicketWithIp: ', ticket.getBytes())

        ticket.cancel()
    }

    {
        const ticket = await client.auth.getSessionTicket('test')
        console.log('getSessionTicketWithSteamId: ', ticket)
        console.log('getAuthTicketForWebApi: ', ticket.getBytes())

        ticket.cancel()
    }
})()
