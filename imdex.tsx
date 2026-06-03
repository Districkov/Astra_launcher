import GTAV334 from "./GTAV-33-4.png";
import image from "./image.svg";
import polygon12 from "./polygon-12.svg";
import polygon13 from "./polygon-13.svg";
import star23 from "./star-23.svg";
import star24 from "./star-24.svg";
import subtract from "./subtract.svg";
import vector from "./vector.svg";
import vector2 from "./vector-2.svg";
import vector3 from "./vector-3.svg";
import vector4 from "./vector-4.svg";
import vector5 from "./vector-5.svg";
import vector6 from "./vector-6.svg";

const menuItems = [
  {
    id: "play",
    label: "Играть",
    icon: polygon12,
    iconAlt: "Играть",
    active: true,
    textClassName: "text-white",
    iconClassName: "top-[171px] left-[34px] w-2.5 h-3",
    textPositionClassName: "top-[168px] left-14",
  },
  {
    id: "shop",
    label: "Магазин",
    icon: vector,
    iconAlt: "Магазин",
    active: false,
    textClassName: "text-[#ffffff4c]",
    iconClassName: "w-[96.85%] top-[calc(50.00%_-_116px)] left-[3.15%] h-4",
    textPositionClassName: "top-[203px] left-14",
  },
  {
    id: "news",
    label: "Новости",
    icon: vector2,
    iconAlt: "Новости",
    active: false,
    textClassName: "text-[#ffffff4c]",
    iconClassName: "w-[96.85%] top-[calc(50.00%_-_22px)] left-[3.15%] h-4",
    textPositionClassName: "top-[297px] left-14",
  },
  {
    id: "forum",
    label: "Форум",
    icon: vector3,
    iconAlt: "Форум",
    active: false,
    textClassName: "text-[#ffffff4c]",
    iconClassName: "w-[96.86%] top-[calc(50.00%_+_14px)] left-[3.14%] h-[15px]",
    textPositionClassName: "top-[332px] left-14",
  },
  {
    id: "discord",
    label: "Discord",
    icon: vector4,
    iconAlt: "Discord",
    active: false,
    textClassName: "text-[#ffffff4c]",
    iconClassName: "w-[96.77%] top-[calc(50.00%_+_72px)] left-[3.23%] h-[11px]",
    textPositionClassName: "top-[388px] left-14",
  },
  {
    id: "settings",
    label: "Настройки",
    icon: vector5,
    iconAlt: "Настройки",
    active: false,
    textClassName: "text-[#ffffff4c]",
    iconClassName:
      "w-[96.77%] top-[calc(50.00%_+_106px)] left-[3.23%] h-[13px]",
    textPositionClassName: "top-[423px] left-14",
  },
];

export const Frame = (): JSX.Element => {
  return (
    <main
      className="relative w-[960px] h-[640px] rounded-[5px] overflow-hidden bg-[url(/rectangle-40403.png)] bg-cover bg-[50%_50%]"
      aria-label="ASTRA launcher"
    >
      <img
        className="top-[410px] left-[702px] w-[258px] h-[230px] absolute aspect-[1] pointer-events-none select-none"
        alt=""
        src={star23}
        aria-hidden="true"
      />
      <img
        className="top-0 left-[106px] w-[323px] h-[156px] absolute aspect-[1] pointer-events-none select-none"
        alt=""
        src={star24}
        aria-hidden="true"
      />
      <img
        className="absolute top-0 left-[504px] w-[456px] h-[400px] pointer-events-none select-none"
        alt=""
        src={subtract}
        aria-hidden="true"
      />
      <div
        className="absolute top-[108px] left-[230px] w-[662px] h-[400px] flex pointer-events-none"
        aria-hidden="true"
      >
        <div className="-mt-px w-[660px] h-[400px] -ml-px [-webkit-text-stroke:1px_#ffffff0d] [font-family:'Armor_Piercing_2.0_BB-Regular',Helvetica] font-normal text-transparent text-[400px] tracking-[-8.00px] leading-[normal] whitespace-nowrap">
          ASTRA
        </div>
      </div>
      <img
        className="absolute top-4 left-[158px] w-[802px] h-[624px] aspect-[1.21] object-cover pointer-events-none select-none"
        alt=""
        src={GTAV334}
        aria-hidden="true"
      />
      <img
        className="absolute top-[282px] left-0 w-[642px] h-[358px] pointer-events-none select-none"
        alt=""
        src={image}
        aria-hidden="true"
      />
      <aside className="absolute top-0 left-px w-[157px] h-[640px] bg-[#1b1b1b] rounded-[5px_0px_0px_5px]">
        <div className="absolute top-[26px] left-11 w-[72px] h-10">
          <div className="absolute top-0 left-0.5 [text-shadow:0px_0px_142.5px_#ffffff] [font-family:'Armor_Piercing_2.0_BB-Regular',Helvetica] font-normal text-white text-[40px] tracking-[-0.80px] leading-[normal] whitespace-nowrap">
            ASTRA
          </div>
          <div className="absolute top-[33px] left-0 w-[70px] h-[3px] bg-[#f64a46] rounded-[21px]" />
        </div>
        <p className="top-[106px] left-[50px] [font-family:'Proxima_Nova-Bold',Helvetica] font-bold text-transparent text-sm tracking-[-0.28px] absolute leading-[normal]">
          <span className="text-[#ffffff4c] tracking-[-0.04px]">Онлайн:</span>
          <span className="text-white tracking-[-0.04px]"> 150</span>
        </p>
        <div
          className="top-[108px] left-[31px] w-3 h-3 bg-[#15ff002e] rounded-md absolute aspect-[1]"
          aria-hidden="true"
        />
        <div
          className="top-[110px] left-[33px] w-2 h-2 bg-[#15ff00] rounded absolute aspect-[1]"
          aria-hidden="true"
        />
        <nav aria-label="Основная навигация">
          <div className="absolute top-40 left-0 w-[158px] h-[34px] bg-[linear-gradient(90deg,rgba(246,74,70,0.5)_0%,rgba(0,0,0,0.1)_100%),linear-gradient(0deg,rgba(217,217,217,0)_0%,rgba(217,217,217,0)_100%)] bg-colors-miscellaneous-sidebar-shadow-drag-over" />
          <ul className="m-0 p-0 list-none">
            {menuItems.map((item) => (
              <li key={item.id}>
                <button
                  type="button"
                  aria-current={item.active ? "page" : undefined}
                  className="all-[unset] cursor-pointer"
                >
                  <img
                    className={`absolute ${item.iconClassName} aspect-[1] pointer-events-none select-none`}
                    alt=""
                    src={item.icon}
                    aria-hidden="true"
                  />
                  <span
                    className={`${item.textPositionClassName} [font-family:'Proxima_Nova-Semibold',Helvetica] font-normal ${item.textClassName} text-[15px] tracking-[-0.30px] whitespace-nowrap absolute leading-[normal]`}
                  >
                    {item.label}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        </nav>
        <div className="absolute top-[576px] left-[57px] [font-family:'Proxima_Nova-Bold',Helvetica] font-bold text-white text-sm tracking-[-0.28px] leading-[normal]">
          r4kuzan
        </div>
        <button
          type="button"
          className="all-[unset] cursor-pointer top-[593px] left-[57px] [font-family:'Proxima_Nova-Bold',Helvetica] font-bold text-[#ffffff4c] text-sm tracking-[-0.28px] absolute leading-[normal]"
        >
          Выйти
        </button>
        <img
          className="absolute w-[97.50%] top-[calc(50.00%_+_261px)] left-[2.50%] h-[25px] aspect-[0.91] pointer-events-none select-none"
          alt=""
          src={vector6}
          aria-hidden="true"
        />
      </aside>
      <div
        className="absolute top-[528px] left-[595px] w-[210px] h-[70px] rounded-[1000px_0px_0px_1000px] bg-[linear-gradient(90deg,rgba(0,0,0,0)_0%,rgba(246,74,70,0.21)_100%),linear-gradient(0deg,rgba(246,74,70,0)_0%,rgba(246,74,70,0)_100%)] pointer-events-none"
        aria-hidden="true"
      />
      <button
        type="button"
        className="absolute top-[528px] left-[750px] w-[210px] h-[70px] bg-[#f64a46] rounded-[1000px_0px_0px_1000px] cursor-pointer"
        aria-label="Играть"
      >
        <span className="top-[550px] left-[835px] [font-family:'Proxima_Nova-Semibold',Helvetica] font-normal text-white text-[22px] tracking-[-0.44px] absolute leading-[normal]">
          Играть
        </span>
        <img
          className="top-[555px] left-[811px] w-4 h-[17px] absolute aspect-[1] pointer-events-none select-none"
          alt=""
          src={polygon13}
          aria-hidden="true"
        />
      </button>
    </main>
  );
};
