# Menús, HUD en partida, avisos del altar y mensajes de muerte (Español de España — es-ES)

# Menús (14)
menu-play = Jugar Turno
menu-practice = Practicar con bots
menu-workshop = Taller
menu-tallies = Fichas de latón (1931)
menu-settings = Ajustes
menu-language = Idioma
menu-audio = Audio y radio
menu-controls = Controles
menu-video = Vídeo
menu-resume = Reanudar Turno
menu-leave = Abandonar Turno
menu-quit = Salir a la superficie
menu-choose-crew = Elige tu cuadrilla
menu-choose-class = Selecciona clase minera

# HUD en partida (17)
hud-bank = Banco: ${ $cash } / ${ $target }
hud-ore-carried = { $count -> [one] { $count } unidad de mena *[other] { $count } unidades de mena }
hud-gold-carried = { $count -> [one] { $count } lingote de oro *[other] { $count } lingotes de oro }
hud-diamond-carried = { $count -> [one] { $count } gema de diamante *[other] { $count } gemas de diamante }
hud-tally-carried = Ficha de latón #{ $number }
hud-seismo-level = Sismógrafo: { $percent } %
hud-drop-in-timer = { $seconds -> [one] Descenso de jaula en { $seconds } segundo *[other] Descenso de jaula en { $seconds } segundos }
hud-wake-timer = { $seconds -> [one] Colapso en { $seconds } segundo *[other] Colapso en { $seconds } segundos }
hud-buckets-stopping = { $seconds -> [one] Las líneas de cangilones paran en { $seconds } segundo *[other] Las líneas de cangilones paran en { $seconds } segundos }
hud-buckets-halted = ¡Líneas de cangilones detenidas — subid el botín a pie!
hud-fame-active = FAMA: ¡{ $team } superó el 60 % de la meta!
hud-stolen-bonus = Botín robado ×1,25 (+${ $cash })
hud-loot-bag-pickup = Bolsa de botín recuperada (+${ $cash })
hud-respawn-timer = { $seconds -> [one] Reapareciendo en { $seconds } segundo... *[other] Reapareciendo en { $seconds } segundos... }
hud-geophone-contact = { $count -> [one] Geófono: { $count } contacto acústico ({ $distance } m) *[other] Geófono: { $count } contactos acústicos ({ $distance } m) }
hud-offering-prompt = Ofrecer ${ $cost } de oro en el altar (-2,5 % Sismógrafo)
hud-tally-prompt = Devolver Ficha de latón al altar (-8,0 % Sismógrafo)

# Avisos y errores del altar (6)
altar-err-closed = El Durmiente ha despertado. El altar está sellado hasta el final del Turno.
altar-err-not-enough-gold = La Ofrenda requiere ${ $required } en oro (llevas ${ $carried }).
altar-err-cooling-down = { $seconds -> [one] Altar enfriándose. Listo en { $seconds } segundo (tick { $ready_at_tick }). *[other] Altar enfriándose. Listo en { $seconds } segundos (tick { $ready_at_tick }). }
altar-err-tally-limit = { $max -> [one] Límite del Turno alcanzado: máximo { $max } Ficha de latón por Turno. *[other] Límite del Turno alcanzado: máximo { $max } Fichas de latón por Turno. }
altar-offering-success = { $team } pagó una Ofrenda de ${ $gold } (-{ $reduction } ptos. en el Sismógrafo).
altar-tally-success = { $team } devolvió la Ficha de latón #{ $number } (-{ $reduction } ptos. en el Sismógrafo).

# Mensajes de muerte (10)
death-lava = A { $player } se lo tragó la lava fundida.
death-electric = { $player } se electrocutó con un tendido activo.
death-explosive = { $player } cayó en una explosión de demolición.
death-fall = { $player } cayó por un pozo profundo.
death-rockfall = A { $player } lo aplastó un derrumbe tras sonar los Silbadores.
death-shove-lava = { $attacker } empujó a { $victim } a la lava con el pico.
death-shove-fall = { $attacker } empujó a { $victim } al fondo del pozo.
death-shove-electric = { $attacker } empujó a { $victim } contra un bloque eléctrico.
death-collapse = { $player } quedó atrás cuando el Durmiente selló la mina.
death-dropped-loot = Soltó una bolsa de botín con ${ $cash } y { $ore } de mena.
