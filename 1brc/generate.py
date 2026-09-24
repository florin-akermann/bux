"""Write a file of measurements for `1brc/main.bx`: `uv run 1brc/generate.py <rows> <path>`.

Each line is `<station>;<reading>`, with a station from a fixed list and a reading from -99.9 to
99.9 with one decimal. The file is written in blocks, so a billion rows need no more memory than
one block holds.
"""

import random
import sys

STATIONS = (
    "Abha", "Abidjan", "Abéché", "Accra", "Addis Ababa", "Adelaide", "Aden", "Ahvaz",
    "Albuquerque", "Alexandra", "Alexandria", "Algiers", "Alice Springs", "Almaty", "Amsterdam",
    "Anadyr", "Anchorage", "Andorra la Vella", "Ankara", "Antananarivo", "Antsiranana", "Arkhangelsk",
    "Ashgabat", "Asmara", "Assab", "Astana", "Athens", "Atlanta", "Auckland", "Austin", "Baghdad",
    "Baguio", "Baku", "Baltimore", "Bamako", "Bangkok", "Bangui", "Banjul", "Barcelona", "Bata",
    "Batumi", "Beijing", "Beirut", "Belgrade", "Belize City", "Benghazi", "Bergen", "Berlin",
    "Bilbao", "Birao", "Bishkek", "Bissau", "Blantyre", "Bloemfontein", "Boise", "Bordeaux",
    "Bosaso", "Boston", "Bouaké", "Bratislava", "Brazzaville", "Bridgetown", "Brisbane",
    "Brussels", "Bucharest", "Budapest", "Bujumbura", "Bulawayo", "Burnie", "Busan", "Cabo San Lucas",
    "Cairns", "Cairo", "Calgary", "Canberra", "Cape Town", "Changsha", "Charlotte", "Chiang Mai",
    "Chicago", "Chihuahua", "Chittagong", "Chișinău", "Chongqing", "Christchurch", "City of San Marino",
    "Colombo", "Columbus", "Conakry", "Copenhagen", "Cotonou", "Cracow", "Da Lat", "Da Nang",
    "Dakar", "Dallas", "Damascus", "Dampier", "Dar es Salaam", "Darwin", "Denpasar", "Denver",
    "Detroit", "Dhaka", "Dikson", "Dili", "Djibouti", "Dodoma", "Dolisie", "Douala", "Dubai",
    "Dublin", "Dunedin", "Durban", "Dushanbe", "Edinburgh", "Edmonton", "El Paso", "Entebbe",
    "Erbil", "Erzurum", "Fairbanks", "Fianarantsoa", "Flores", "Frankfurt", "Fresno",
    "Fukuoka", "Gaborone", "Gabès", "Gangtok", "Garissa", "Garoua", "George Town", "Ghanzi",
    "Gjoa Haven", "Guadalajara", "Guangzhou", "Guatemala City", "Halifax", "Hamburg", "Hamilton",
    "Hanga Roa", "Hanoi", "Harare", "Harbin", "Hargeisa", "Hat Yai", "Havana", "Helsinki",
    "Heraklion", "Hiroshima", "Ho Chi Minh City", "Hobart", "Hong Kong", "Honiara", "Honolulu",
    "Houston", "Ifrane", "Indianapolis", "Iqaluit", "Irkutsk", "Istanbul", "İzmir", "Jacksonville",
    "Jakarta", "Jayapura", "Jerusalem", "Johannesburg", "Jos", "Juba", "Kabul", "Kampala",
    "Kandi", "Kankan", "Kano", "Kansas City", "Karachi", "Karonga", "Kathmandu", "Khartoum",
    "Kingston", "Kinshasa", "Kolkata", "Kuala Lumpur", "Kumasi", "Kunming", "Kuopio", "Kuwait City",
    "Kyiv", "Kyoto", "La Ceiba", "La Paz", "Lagos", "Lahore", "Lake Havasu City", "Lake Tekapo",
    "Las Palmas de Gran Canaria", "Las Vegas", "Launceston", "Lhasa", "Libreville", "Lisbon",
    "Livingstone", "Ljubljana", "Lodwar", "Lomé", "London", "Los Angeles", "Louisville",
    "Luanda", "Lubumbashi", "Lusaka", "Luxembourg City", "Lviv", "Lyon", "Madrid", "Mahajanga",
    "Makassar", "Makurdi", "Malabo", "Malé", "Managua", "Manama", "Mandalay", "Mango", "Manila",
    "Maputo", "Marrakesh", "Marseille", "Maun", "Medan", "Mek'ele", "Melbourne", "Memphis",
    "Mexicali", "Mexico City", "Miami", "Milan", "Milwaukee", "Minneapolis", "Minsk", "Mogadishu",
    "Mombasa", "Monaco", "Moncton", "Monterrey", "Montreal", "Moscow", "Mumbai", "Murmansk",
    "Muscat", "Mzuzu", "N'Djamena", "Naha", "Nairobi", "Nakhon Ratchasima", "Napier", "Napoli",
    "Nashville", "Nassau", "Ndola", "New Delhi", "New Orleans", "New York City", "Ngaoundéré",
    "Niamey", "Nicosia", "Niigata", "Nouadhibou", "Nouakchott", "Novosibirsk", "Nuuk", "Odesa",
    "Odienné", "Oklahoma City", "Omaha", "Oranjestad", "Oslo", "Ottawa", "Ouagadougou",
    "Ouahigouya", "Ouarzazate", "Oulu", "Palembang", "Palermo", "Palm Springs", "Palmerston North",
    "Panama City", "Parakou", "Paris", "Perth", "Petropavlovsk-Kamchatsky", "Philadelphia",
    "Phnom Penh", "Phoenix", "Pittsburgh", "Podgorica", "Pointe-Noire", "Pontianak", "Port Moresby",
    "Port Sudan", "Port Vila", "Port-Gentil", "Portland (OR)", "Porto", "Prague", "Praia",
    "Pretoria", "Pyongyang", "Rabat", "Rangpur", "Reggane", "Reykjavík", "Riga", "Riyadh", "Rome",
    "Roseau", "Rostov-on-Don", "Sacramento", "Saint Petersburg", "Saint-Pierre", "Salt Lake City",
    "San Antonio", "San Diego", "San Francisco", "San Jose", "San José", "San Juan", "San Salvador",
    "Sana'a", "Santo Domingo", "Sapporo", "Sarajevo", "Saskatoon", "Seattle", "Ségou", "Seoul",
    "Seville", "Shanghai", "Singapore", "Skopje", "Sochi", "Sofia", "Sokoto", "Split", "St. John's",
    "St. Louis", "Stockholm", "Surabaya", "Suva", "Suwałki", "Sydney", "Tabora", "Tabriz",
    "Taipei", "Tallinn", "Tamale", "Tamanrasset", "Tampa", "Tashkent", "Tauranga", "Tbilisi",
    "Tegucigalpa", "Tehran", "Tel Aviv", "Thessaloniki", "Thiès", "Tijuana", "Timbuktu", "Tirana",
    "Toamasina", "Tokyo", "Toliara", "Toluca", "Toronto", "Tripoli", "Tromsø", "Tucson", "Tunis",
    "Ulaanbaatar", "Upington", "Ürümqi", "Vaduz", "Valencia", "Valletta", "Vancouver", "Veracruz",
    "Vienna", "Vientiane", "Villahermosa", "Vilnius", "Virginia Beach", "Vladivostok", "Warsaw",
    "Washington, D.C.", "Wau", "Wellington", "Whitehorse", "Wichita", "Willemstad", "Winnipeg",
    "Wrocław", "Xi'an", "Yakutsk", "Yangon", "Yaoundé", "Yellowknife", "Yerevan", "Yinchuan",
    "Zagreb", "Zanzibar City", "Zürich",
)

ROWS_IN_A_BLOCK = 100_000


def main() -> int:
    match sys.argv[1:]:
        case [rows, path] if rows.isdigit():
            write(int(rows), path)
            return 0
        case _:
            print("usage: uv run 1brc/generate.py <rows> <path>", file=sys.stderr)
            return 2


def write(rows: int, path: str) -> None:
    with open(path, "w", encoding="utf-8", newline="\n") as file:
        left = rows
        while left > 0:
            count = min(left, ROWS_IN_A_BLOCK)
            file.write("".join(line() for _ in range(count)))
            left -= count


def line() -> str:
    tenths = random.randint(-999, 999)
    sign = "-" if tenths < 0 else ""
    whole, tenth = divmod(abs(tenths), 10)
    return f"{random.choice(STATIONS)};{sign}{whole}.{tenth}\n"


if __name__ == "__main__":
    sys.exit(main())
