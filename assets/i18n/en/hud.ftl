# Menús, HUD en partida, avisos del altar y mensajes de muerte (Inglés — idioma fuente canónico)

# Menús (14)
menu-play = Play Shift
menu-practice = Practice with Bots
menu-workshop = Workshop
menu-tallies = Brass Tallies (1931)
menu-settings = Settings
menu-language = Language
menu-audio = Audio & Radio
menu-controls = Controls
menu-video = Video
menu-resume = Resume Shift
menu-leave = Leave Shift
menu-quit = Quit to Surface
menu-choose-crew = Choose Your Crew
menu-choose-class = Select Mining Class

# HUD en partida (17)
hud-bank = Bank: ${ $cash } / ${ $target }
hud-ore-carried = { $count -> [one] { $count } ore unit *[other] { $count } ore units }
hud-gold-carried = { $count -> [one] { $count } gold bar *[other] { $count } gold bars }
hud-diamond-carried = { $count -> [one] { $count } diamond gem *[other] { $count } diamond gems }
hud-tally-carried = Brass tally #{ $number }
hud-seismo-level = Seismograph: { $percent }%
hud-drop-in-timer = { $seconds -> [one] Cage drop in { $seconds } second *[other] Cage drop in { $seconds } seconds }
hud-wake-timer = { $seconds -> [one] Collapse in { $seconds } second *[other] Collapse in { $seconds } seconds }
hud-buckets-stopping = { $seconds -> [one] Bucket lines stop in { $seconds } second *[other] Bucket lines stop in { $seconds } seconds }
hud-buckets-halted = Bucket lines halted — haul on foot!
hud-fame-active = FAME: { $team } passed 60% of the goal!
hud-stolen-bonus = Stolen loot ×1.25 (+${ $cash })
hud-loot-bag-pickup = Recovered loot bag (+${ $cash })
hud-respawn-timer = { $seconds -> [one] Respawning in { $seconds } second... *[other] Respawning in { $seconds } seconds... }
hud-geophone-contact = { $count -> [one] Geophone: { $count } acoustic contact ({ $distance }m) *[other] Geophone: { $count } acoustic contacts ({ $distance }m) }
hud-offering-prompt = Offer ${ $cost } gold at the altar (-2.5% Seismograph)
hud-tally-prompt = Return Brass tally to the altar (-8.0% Seismograph)

# Avisos y errores del altar (6)
altar-err-closed = The Sleeper is awake. The altar is sealed until the Shift ends.
altar-err-not-enough-gold = Offering requires ${ $required } in gold (you carry ${ $carried }).
altar-err-cooling-down = { $seconds -> [one] Altar cooling down. Ready in { $seconds } second (tick { $ready_at_tick }). *[other] Altar cooling down. Ready in { $seconds } seconds (tick { $ready_at_tick }). }
altar-err-tally-limit = { $max -> [one] Shift limit reached: maximum { $max } Brass tally per Shift. *[other] Shift limit reached: maximum { $max } Brass tallies per Shift. }
altar-offering-success = { $team } paid an Offering of ${ $gold } (-{ $reduction } pts on the Seismograph).
altar-tally-success = { $team } returned Brass tally #{ $number } (-{ $reduction } pts on the Seismograph).

# Mensajes de muerte (10)
death-lava = { $player } was swallowed by molten lava.
death-electric = { $player } was electrocuted by a live conduit.
death-explosive = { $player } was caught in a demolition blast.
death-fall = { $player } fell down a deep shaft.
death-rockfall = { $player } was crushed by a cave-in after the Whistlers sounded.
death-shove-lava = { $attacker } shoved { $victim } into the lava with a pickaxe.
death-shove-fall = { $attacker } shoved { $victim } down the shaft.
death-shove-electric = { $attacker } shoved { $victim } onto an electric block.
death-collapse = { $player } was left behind when the Sleeper sealed the mine.
death-dropped-loot = Dropped a loot bag worth ${ $cash } and { $ore } ore.
