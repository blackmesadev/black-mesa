package admin

var BOT_ADMINS = [3]string{"206309860038410240", "198067780455497738", "96269247411400704"} // Tyler#0911, Flashy#1984, LewisTehMinerz#1337

func IsBotAdmin(id string) bool {
	for _, admin := range BOT_ADMINS {
		if admin == id {
			return true
		}
	}
	return false
}
